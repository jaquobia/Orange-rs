use std::collections::HashMap;
use std::f32::consts;
use std::ops::{Mul, Rem};
use ultraviolet::{Mat4, Rotor3, Vec2, Vec3};

use crate::direction::{Direction, DIRECTIONS};

pub struct ModelPoly<const SIZE:  usize> {
    pub pos: [Vec3; SIZE],
    pub normal: Vec3,
    pub color: Vec3,
    pub u: Vec2,
    pub v: Vec2,
    pub cullface: Option<Direction>,
    pub texture: String,
    pub tint_index: i32,
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

fn get_face_vertices_on_cuboid(face: Direction, points: &[Vec3; 8]) -> [Vec3; 4] {
    match face {
        Direction::North => [
            points[0],
            points[1],
            points[2],
            points[3],
        ],
        Direction::South => [
            points[4],
            points[5],
            points[6],
            points[7],
        ],
        Direction::East => [
            points[5],
            points[0],
            points[7],
            points[2],
        ],
        Direction::West => [
            points[1],
            points[4],
            points[3],
            points[6],
        ],
        Direction::Up => [
            points[5],
            points[4],
            points[0],
            points[1],
        ],
        Direction::Down => [
            points[6],
            points[7],
            points[3],
            points[2],
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
    pub fn with_texture(self, texture_variable: impl Into<String>, texture_id: impl Into<String>) -> Self {
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

    fn rotate_points(points: &mut [Vec3; 8], rescale: bool, angle: f32, axis: u8, origin: Vec3) {
        let angle = angle.to_radians();
        let axis = match axis {
            0 => Vec3::unit_x(),
            1 => Vec3::unit_y(),
            2 => Vec3::unit_z(),
            _ => Vec3::zero(),
        };
        let rotation = Mat4::from_rotation_around(axis.xyzw(), angle).extract_rotation().normalized();
        // the function provided by minecraft's wiki under the sapling example, but doesn't work: https://minecraft.fandom.com/wiki/Tutorials/Models#Block_models
        // let scale = if *rescale { 1.0 + 1.0 / (angle.cos() - 1.0) } else { 1.0 };
        let scale = if rescale { let t = 0.5 - 0.5*(4.0*angle).sin(); (1.0 - t) + t * 2.0_f32.sqrt() } else { 1.0 };

        let center = origin * (1.0 / 16.0);
        let scale = ((Vec3::one() - axis) * scale) + axis;
        for i in 0..8 {
            points[i] = points[i] - center;

            points[i].x *= scale.x;
            points[i].y *= scale.y;
            points[i].z *= scale.z;
        }
        rotation.rotate_vecs(points);
        for i in 0..8 {
            points[i] += center;
        }
    }

    fn rotate_direction(dir: Option<Direction>, angle: f32) -> Option<Direction> {
        let angle = angle.rem(360.0);
        match dir {
            Some(dir) => {
                if angle == 0.0 {
                    Some(dir)
                } else if angle == 90.0 {
                    Some(dir.ccw())
                } else if angle == 180.0 {
                    Some(dir.reverse_horizontal())
                } else if angle == 270.0 {
                    Some(dir.cw())
                } else { None }
            },
            None => None,
        }
    }

    pub fn bake(self) -> BakedModel {
        self.bake_with_rotate(None)
    }

    pub fn bake_with_rotate(self, variant_rotation: Option<VoxelRotation>) -> BakedModel {
        let mut quads: Vec<ModelShape> = vec![];
        let textures = self.textures;
        for element in &self.elements {
            let min_pos = element.from * (1.0 / 16.0);
            let max_pos = element.to * (1.0 / 16.0);

            let mut points = &mut [
                Vec3::new(min_pos.x, max_pos.y, min_pos.z), // 0
                Vec3::new(min_pos.x, max_pos.y, max_pos.z), // 1
                Vec3::new(min_pos.x, min_pos.y, min_pos.z), // 2
                Vec3::new(min_pos.x, min_pos.y, max_pos.z), // 3
                Vec3::new(max_pos.x, max_pos.y, max_pos.z), // 4
                Vec3::new(max_pos.x, max_pos.y, min_pos.z), // 5
                Vec3::new(max_pos.x, min_pos.y, max_pos.z), // 6
                Vec3::new(max_pos.x, min_pos.y, min_pos.z), // 7
            ];

            if let Some(VoxelRotation{rescale , angle, axis, origin }) = &element.rotation {
                Self::rotate_points(points, *rescale, *angle, *axis, *origin);
            }
            let mut variant_rotation_angle = 0.0;
            if let Some(VoxelRotation{rescale , angle, axis, origin }) = variant_rotation {
                Self::rotate_points(points, rescale, angle, axis, origin);
                variant_rotation_angle = angle;
            }

            for (index, face) in element.faces.iter().enumerate() {
                if let Some(face) = face {
                    let face_direction = DIRECTIONS[index];
                    let pos = get_face_vertices_on_cuboid(face_direction, points);
                    let (u, v) = if let Some(uv) = face.uv {
                        (uv[0], uv[1])
                    } else {
                        ((0.0, 0.0).into(), (16.0, 16.0).into())
                    };
                    let cullface = Self::rotate_direction(face.cullface, variant_rotation_angle);
                    quads.push( ModelShape::Quad { quad: ModelQuad { pos, u, v, texture: face.texture_variable.clone(), color: (1.0, 1.0, 1.0).into(), cullface, normal: face_direction.get_float_vector(), tint_index: face.tint_index} } );
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
    tint_index: i32,
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
            tint_index: -1,
        }
    }
    pub fn with_uv(self, u: impl Into<Vec2>, v: impl Into<Vec2>) -> Self {
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
    pub fn with_tint(mut self, tint: i32) -> Self {
        self.tint_index = tint;
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