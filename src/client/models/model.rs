use std::collections::HashMap;
use ultraviolet::{Vec2, Vec3};

use crate::direction::{Direction, DIRECTIONS};

pub struct ModelPoly<const SIZE:  usize> {
    pub pos: [Vec3; SIZE],
    pub normal: Vec3,
    pub color: Vec3,
    pub u: Vec2,
    pub v: Vec2,
    pub cullface: Option<Direction>,
    pub texture: String,
}

type ModelQuad = ModelPoly<4>;
type ModelTriangle = ModelPoly<3>;

pub enum ModelShape {
    Quad { quad: ModelQuad },
    Triangle { triangle: ModelTriangle },
}

impl ModelShape {
    pub fn num_pos(&self) -> usize {
        match self {
            ModelShape::Quad { quad } => { quad.pos.len() },
            ModelShape::Triangle { triangle } => { triangle.pos.len() },
        }
    }
    pub fn pos(&self, index: usize) -> Vec3 {
        match self {
            ModelShape::Quad { quad } => { quad.pos[index] },
            ModelShape::Triangle { triangle } => { triangle.pos[index] },
        }
    }
    pub fn normal(&self) -> Vec3 {
        match self {
            ModelShape::Quad { quad } => { quad.normal },
            ModelShape::Triangle { triangle } => { triangle.normal },
        }
    }
    pub fn color(&self) -> Vec3 {
        match self {
            ModelShape::Quad { quad } => { quad.color },
            ModelShape::Triangle { triangle } => { triangle.color },
        }
    }
    pub fn uv(&self) -> [Vec2; 2] {
        match self {
            ModelShape::Quad { quad } => { [quad.u, quad.v] },
            ModelShape::Triangle { triangle } => { [triangle.u, triangle.v] },
        }
    }
    pub fn cullface(&self) -> Option<Direction> {
        match self {
            ModelShape::Quad { quad } => { quad.cullface },
            ModelShape::Triangle { triangle } => { triangle.cullface },
        }
    }
    pub fn texture(&self) -> &String {
        match self {
            ModelShape::Quad { quad } => { &quad.texture },
            ModelShape::Triangle { triangle } => { &triangle.texture },
        }
    }
}

pub struct BakedModel {
    quads: Vec<ModelShape>,
    textures: HashMap<String, String>,
}

impl BakedModel {
    pub fn new() -> Self {
        Self {
            quads: vec![],
            textures: HashMap::new(),
        }
    }

    pub fn shapes(&self) -> &Vec<ModelShape> {
        &self.quads
    }
    pub fn textures(&self) -> &HashMap<String, String> {
        &self.textures
    }
}

#[derive(Clone)]
pub struct VoxelModel {
    textures: HashMap<String, String>,
    elements: Vec<VoxelElement>,
    ambient_occlusion: bool,
}

fn get_face_vertices_on_cuboid(pos_min: &Vec3, pos_max: &Vec3, face: Direction) -> [Vec3; 4] {
    match face {
        Direction::North => [
            Vec3::new(pos_min.x, pos_max.y, pos_min.z),
            Vec3::new(pos_min.x, pos_max.y, pos_max.z),
            Vec3::new(pos_min.x, pos_min.y, pos_min.z),
            Vec3::new(pos_min.x, pos_min.y, pos_max.z),
        ],
        Direction::South => [
            Vec3::new(pos_max.x, pos_max.y, pos_max.z),
            Vec3::new(pos_max.x, pos_max.y, pos_min.z),
            Vec3::new(pos_max.x, pos_min.y, pos_max.z),
            Vec3::new(pos_max.x, pos_min.y, pos_min.z),
        ],
        Direction::East => [
            Vec3::new(pos_max.x, pos_max.y, pos_min.z),
            Vec3::new(pos_min.x, pos_max.y, pos_min.z),
            Vec3::new(pos_max.x, pos_min.y, pos_min.z),
            Vec3::new(pos_min.x, pos_min.y, pos_min.z),
        ],
        Direction::West => [
            Vec3::new(pos_min.x, pos_max.y, pos_max.z),
            Vec3::new(pos_max.x, pos_max.y, pos_max.z),
            Vec3::new(pos_min.x, pos_min.y, pos_max.z),
            Vec3::new(pos_max.x, pos_min.y, pos_max.z),
        ],
        Direction::Up => [
            Vec3::new(pos_max.x, pos_max.y, pos_min.z),
            Vec3::new(pos_max.x, pos_max.y, pos_max.z),
            Vec3::new(pos_min.x, pos_max.y, pos_min.z),
            Vec3::new(pos_min.x, pos_max.y, pos_max.z),
        ],
        Direction::Down => [
            Vec3::new(pos_max.x, pos_min.y, pos_max.z),
            Vec3::new(pos_max.x, pos_min.y, pos_min.z),
            Vec3::new(pos_min.x, pos_min.y, pos_max.z),
            Vec3::new(pos_min.x, pos_min.y, pos_min.z),
        ],
    }
}

impl VoxelModel {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            elements: Vec::new(),
            ambient_occlusion: true,
        }
    }
    pub fn from_template(template: &VoxelModel) -> Self {
        template.clone()
    }
    pub fn with_texture(mut self, texture_variable: impl Into<String>, texture_id: impl Into<String>) -> Self {
        self.with_texture_inner(texture_variable.into(), texture_id.into())
    }
    fn with_texture_inner(mut self, texture_variable: String, texture_id: String) -> Self {
        self.textures.insert(texture_variable, texture_id);
        self
    }
    pub fn with_element(mut self, element: VoxelElement) -> Self {
        self.elements.push(element);
        self
    }
    pub fn with_ambient_occlusion(mut self, ambient_occlusion: bool) -> Self {
        self.ambient_occlusion = ambient_occlusion;
        self
    }

    pub fn bake(self) -> BakedModel {
        let mut quads: Vec<ModelShape> = vec![];
        let textures = self.textures;
        for element in &self.elements {
            for (index, face) in element.faces.iter().enumerate() {
                let min_pos = element.from * (1.0 / 16.0);
                let max_pos = element.to * (1.0 / 16.0);
                // TODO: Maybe eventually precalculate the 8 vertices, will make calculating the rotated elements easier
                // let positions: [Vec3; 8] = [
                //
                // ];
                if let Some(face) = face {
                    let face_direction = DIRECTIONS[index];
                    let pos = get_face_vertices_on_cuboid(&min_pos, &max_pos, face_direction);
                    let (u, v) = if let Some(uv) = face.uv {
                        (uv[0], uv[1])
                    } else {
                        ((0.0, 0.0).into(), (0.0, 0.0).into())
                    };
                    quads.push( ModelShape::Quad { quad: ModelQuad { pos, u, v, texture: face.texture_variable.clone(), color: (1.0, 1.0, 1.0).into(), cullface: face.cullface, normal: face_direction.get_float_vector(), } } );
                }
            }
        }
        BakedModel { quads, textures }
    }
}

#[derive(Clone)]
pub struct VoxelElement {
    from: Vec3,
    to: Vec3,
    rotation: Option<VoxelRotation>,
    faces: [Option<VoxelFace>; 6],
    shade: bool,
}

impl VoxelElement {
    pub fn new(from: impl Into<Vec3>, to: impl Into<Vec3>) -> Self {
        Self::new_inner(from.into(), to.into())
    }
    fn new_inner (from: Vec3, to: Vec3) -> Self {
        Self {
            from,
            to,
            rotation: None,
            faces: [None, None, None, None, None, None],
            shade: true,
        }
    }
    pub fn with_face(mut self, face: VoxelFace, side: Direction) -> Self {
        self.faces[side.ordinal()] = Some(face);
        self
    }
    pub fn with_rotation(mut self, rotation: VoxelRotation) -> Self {
        self.rotation = Some(rotation);
        self
    }
    pub fn with_shade(mut self, shade: bool) -> Self {
        self.shade = shade;
        self
    }
}

#[derive(Clone)]
pub struct VoxelFace {
    texture_variable: String,
    uv: Option<[Vec2; 2]>,
    cullface: Option<Direction>,
}

impl VoxelFace {
    pub fn new(texture_variable: impl Into<String>) -> Self {
        Self::new_inner(texture_variable.into())
    }
    pub fn new_inner(texture_variable: String) -> Self {
        Self {
            texture_variable,
            uv: None,
            cullface: None,
        }
    }
    pub fn with_uv(mut self, u: impl Into<Vec2>, v: impl Into<Vec2>) -> Self {
        self.with_uv_inner(u.into(), v.into())
    }
    fn with_uv_inner(mut self, u: Vec2, v: Vec2) -> Self {
        self.uv = Some([u, v]);
        self
    }
    pub fn with_cullface(mut self, cullface: Direction) -> Self {
        self.cullface = Some(cullface);
        self
    }
}

#[derive(Clone)]
pub struct VoxelRotation {
    angle: f32,
    axis: u8,
    origin: Vec3,
    rescale: bool,
}

impl VoxelRotation {
    pub fn new(angle: f32, axis: u8, origin: impl Into<Vec3>, rescale: bool) -> Self {
        Self::new_inner(angle, axis, origin.into(), rescale)
    }
    fn new_inner(angle: f32, axis: u8, origin: Vec3, rescale: bool) -> Self {
        Self {
            angle,
            axis,
            origin,
            rescale,
        }
    }
}