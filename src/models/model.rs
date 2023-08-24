use std::ops::Rem;
use ultraviolet::{Mat4, Vec2, Vec3};
use rustc_hash::FxHashMap as HashMap;

use crate::{direction::{Direction, DIRECTIONS}, sprites::Sprite, minecraft::{identifier::Identifier, registry::SpriteRegister}};

const ONE_SIXTEENTH: f32 = 1.0 / 16.0;

#[derive(Clone, Copy)]
pub struct ModelPoly<const SIZE:  usize> {
    pub pos: [Vec3; SIZE],
    pub uvs: [Vec2; SIZE],
    pub normal: Vec3,
    pub color: Vec3,
    pub cullface: Option<Direction>,
    pub ao_face: Option<Direction>,
    pub tint_index: i32,
}

impl<const SIZE: usize> ModelPoly<SIZE> {

}

pub type ModelQuad = ModelPoly<4>;
pub type ModelTriangle = ModelPoly<3>;

pub struct BakedModel {
    quads: Vec<ModelQuad>,
    ambient_occlusion: bool,
}

impl BakedModel {
    pub fn new() -> Self {
        Self {
            quads: vec![],
            ambient_occlusion: true,
        }
    }

    pub fn shapes(&self) -> &Vec<ModelQuad> {
        &self.quads
    }

    pub fn ambient_occlusion(&self) -> bool { self.ambient_occlusion }

    pub fn combine(&mut self, other: &Self) {
        self.quads.extend(other.quads.clone());
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
        Direction::Up => [ // Before: 5,4,1,0
            points[0],
            points[5],
            points[1],
            points[4],
        ],
        Direction::Down => [ // Before: 6,7,3,2
            points[7],
            points[2],
            points[6],
            points[3],
        ],
    }
}

impl VoxelModel {
    pub fn new() -> Self {
        Self {
            textures: HashMap::default(),
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

    pub fn with_texture_nc(&mut self, texture_variable: impl Into<String>, texture_id: impl Into<String>) {
        self.with_texture_inner_nc(texture_variable.into(), texture_id.into());
    }
    fn with_texture_inner_nc(&mut self, texture_variable: String, texture_id: String) {
        self.textures.insert(texture_variable, texture_id);
    }
    pub fn with_element_nc(&mut self, element: VoxelElement) {
        self.elements.push(element);
    }
    pub fn with_ambient_occlusion_nc(&mut self, ambient_occlusion: bool) {
        self.ambient_occlusion = ambient_occlusion;
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

        let center = origin * ONE_SIXTEENTH;
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

    fn rotate_direction(dir: Option<Direction>, angle: u8) -> Option<Direction> {
        dir.map(|dir| {
            match angle {
                1 => { dir.ccw() },
                2 => { dir.reverse_horizontal() },
                3 => { dir.cw() },
                _ => { dir },
            }
        })
    }

    pub fn flatten_angle_to_index(angle: f32) -> u8 {
        ((angle as u32) / 90u32).rem(4).try_into().unwrap_or(0)
    }

    fn find_texture_in_map(texture_strings: &HashMap<String, String>, mut tex_to_find: String) -> String {

        while tex_to_find.starts_with("#") {
            let a = &texture_strings.get(&tex_to_find[1..]);
            tex_to_find = a.cloned().unwrap_or_else(|| String::from("minecraft:block/missing"));
        }

        return tex_to_find;
    }

    pub fn clear_elements(&mut self) {
        self.elements.clear();
    }

    pub fn bake(self, textures: &SpriteRegister) -> BakedModel {
        self.bake_with_rotate(None, textures)
    }

    pub fn bake_with_rotate(self, variant_rotation: Option<VoxelRotation>, texture_register: &SpriteRegister) -> BakedModel {
        let mut quads = vec![];
        let textures = self.textures;
        for element in &self.elements {
            let min_pos = element.from * ONE_SIXTEENTH;
            let max_pos = element.to * ONE_SIXTEENTH;

            let points = &mut [
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
            let variant_rotation_angle =
            if let Some(VoxelRotation{rescale , angle, axis, origin }) = variant_rotation {
                Self::rotate_points(points, rescale, angle, axis, origin);
                Self::flatten_angle_to_index(angle)
            } else {
                0u8
            };

            for (index, face) in element.faces.iter().enumerate() {
                if let Some(face) = face {
                    let face_direction = DIRECTIONS[index];
                    let pos = get_face_vertices_on_cuboid(face_direction, points);
                    let (uv_min, uv_max) = if let Some(uv) = face.uv {
                        (uv[0], uv[1])
                    } else {
                        // TODO: default uv's to the size of the face
                        ((0.0, 0.0).into(), (16.0, 16.0).into())
                    };
                    let uvs = {
                        let texture = Self::find_texture_in_map(&textures, face.texture_variable.clone());
                        let texture_id = Identifier::from(texture);
                        let (texture_extent_min, texture_extent_max): (Vec2, Vec2) = if let Some(Sprite { parent_texture, uv_min, uv_max }) = texture_register.get(&texture_id) {
                            (*uv_min, *uv_max)
                        } else { 
                            log::warn!("No texture for {}", texture_id);
                            (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)) 
                        };
                        let uv_range = texture_extent_max - texture_extent_min;
                        let (quad_uv_min, quad_uv_max) = (uv_min * ONE_SIXTEENTH, uv_max * ONE_SIXTEENTH);

                        // let (uv_min, uv_max) = (uv_min + uv_range * quad_uv_min, uv_min + uv_range * quad_uv_max);
                        let (uv_min, uv_max) = (texture_extent_min + uv_range * quad_uv_min, texture_extent_min + uv_range * quad_uv_max);

                        [uv_min, (uv_max.x, uv_min.y).into(), (uv_min.x, uv_max.y).into(), uv_max]
                    };
                    let uvs = match face.rotation {
                        1 => { [ uvs[2], uvs[0], uvs[3], uvs[1] ] },
                        2 => { [ uvs[3], uvs[2], uvs[1], uvs[0] ] },
                        3 => { [ uvs[1], uvs[3], uvs[0], uvs[2] ] },
                        _ => { uvs },
                    };
                    let color = (1.0, 1.0, 1.0).into();
                    let cullface = Self::rotate_direction(face.cullface, variant_rotation_angle);
                    let ao_face = if element.rotation.is_some() { None } else { cullface.clone() };
                    let normal = face_direction.get_float_vector();
                    let tint_index = face.tint_index;
                    quads.push( ModelQuad { 
                        pos, 
                        uvs, 
                        color, 
                        cullface, 
                        ao_face, 
                        normal, 
                        tint_index 
                    });
                }
            }
        }
        BakedModel { quads, ambient_occlusion: self.ambient_occlusion }
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

    pub fn with_face_nc(&mut self, face: VoxelFace, side: Direction) {
        self.faces[side.ordinal()] = Some(face);
    }
    pub fn with_rotation_nc(&mut self, rotation: VoxelRotation) {
        self.rotation = Some(rotation);
    }
    pub fn with_shade_nc(&mut self, shade: bool) {
        self.shade = shade;
    }
}

#[derive(Clone)]
pub struct VoxelFace {
    texture_variable: String,
    uv: Option<[Vec2; 2]>,
    cullface: Option<Direction>,
    tint_index: i32,
    /// The rotation (permutation) of the uv on the vertices
    rotation: u8,
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
            rotation: 0,
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

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = VoxelModel::flatten_angle_to_index(rotation);
        self
    }

    pub fn with_uv_nc(&mut self, u: impl Into<Vec2>, v: impl Into<Vec2>) {
        self.with_uv_inner_nc(u.into(), v.into())
    }
    fn with_uv_inner_nc(&mut self, u: Vec2, v: Vec2) {
        self.uv = Some([u, v]);
    }
    pub fn with_cullface_nc(&mut self, cullface: Direction) {
        self.cullface = Some(cullface);
    }
    pub fn with_tint_nc(&mut self, tint: i32) {
        self.tint_index = tint;
    }

    pub fn with_rotation_nc(&mut self, rotation: f32) {
        self.rotation = VoxelModel::flatten_angle_to_index(rotation);
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
