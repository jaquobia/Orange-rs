use std::ops::Add;
use rustc_hash::FxHashMap as HashMap;

use ultraviolet::{IVec3, Vec2, Vec3};
use wgpu::{Device, util::DeviceExt};

use crate::{block::{Block, BlockState}, direction::DIRECTIONS, world::chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE, TLightData}};
use crate::client::models::model::{BakedModel, ModelShape};
use crate::client::textures::TextureObject;
use crate::direction::{DirectionAll, DIRECTIONS_ALL};
use crate::minecraft::identifier::Identifier;
use crate::minecraft::registry::Register;

use crate::world::chunk::TBlockData;
use crate::world::{ChunkStorage, ChunkStorageTrait};

use super::{mesh::Mesh, verticies::TerrainVertex};

enum TessellatorLayer {
    Opaque,
    Transparent,
}

pub struct TerrainTessellator {
    opaque_vertex_buffer: Vec<TerrainVertex>,
    transparent_vertex_buffer: Vec<TerrainVertex>,
    opaque_index_buffer: Vec<u32>,
    transparent_index_buffer: Vec<u32>,
    layer: TessellatorLayer,
}

impl TerrainTessellator {
    /// Construct a new tessellator object
    pub fn new() -> Self {
        Self {
            opaque_vertex_buffer: vec![],
            transparent_vertex_buffer: vec![],
            opaque_index_buffer: vec![],
            transparent_index_buffer: vec![],
            layer: TessellatorLayer::Opaque,
        }
    }

    /// Builder function that tessellates the vertices of a cube into a buffer
    /// # Arguments
    /// * `Pos` The position of the cube
    /// * `color` The color of the cube
    /// * `texture_index` An array of 6 texture indices of terrain.png
    pub fn cuboid(
        &mut self,
        pos: Vec3,
        color: Vec3,
        texture_index: [u32; 6],
        occlusions: &[bool; 6],
    ) -> &mut Self {
        let pos_max = pos + Vec3::new(1.0, 1.0, 1.0);
        let lights = &[0_u32; 8];

        const TEX_SIZE: f32 = 1.0 / (16.0); // 16 textures * 16 pixels per texture, two-fifty-six_inverse

        for dir in &DIRECTIONS {
            if !occlusions[dir.ordinal()] {
                continue;
            }
            let texture_index = texture_index[dir.ordinal()] as u32;
            let texture_x = (texture_index % 16) as f32;
            let texture_y = (texture_index / 16) as f32;
            let uv_min = Vec2::new(texture_x * TEX_SIZE, texture_y * TEX_SIZE);
            let uv_max = uv_min.add(Vec2::new(TEX_SIZE, TEX_SIZE));
            // self.quad(pos, pos_max, color, dir, uv_min, uv_max, lights);
        }
        self
    }

    /// Builder function that tessellates the vertices of a quad facing a cardinal direction into a buffer
    /// # Arguments
    /// `pos_min` the minimum extent of a cube
    /// `pos_max` the maximum extent of the cube
    /// `color` the color of the quad
    /// `direction` the cardinal direction of the quad, determines its vertex orientation and normals
    /// `uv_min` the minimum extent of the uv
    /// `uv_max` the maximum extent of the uv
    /// `lights` the light values of the vertices, currently unused
    pub fn quad(
        &mut self,
        pos: [Vec3; 4],
        lights: [u32; 4],
        color: Vec3,
        normal: Vec3,
        uv_min: Vec2,
        uv_max: Vec2,
        flip_vertex_order: bool,
    ) -> &mut Self {

        let prev_vert_len = self.opaque_vertex_buffer.len() as u32;
        // Top Left
        self.vertex(TerrainVertex::new(pos[0], color, normal, uv_min, lights[0]));
        // Top Right
        self.vertex(TerrainVertex::new(pos[1], color, normal, Vec2::new(uv_max.x, uv_min.y), lights[1]));
        // Bottom Left
        self.vertex(TerrainVertex::new(pos[2], color, normal, Vec2::new(uv_min.x, uv_max.y), lights[2]));
        // Bottom Right
        self.vertex(TerrainVertex::new(pos[3], color, normal, uv_max, lights[3]));

        if flip_vertex_order {
            self.opaque_index_buffer.push(prev_vert_len + 0);
            self.opaque_index_buffer.push(prev_vert_len + 2);
            self.opaque_index_buffer.push(prev_vert_len + 1);
            self.opaque_index_buffer.push(prev_vert_len + 2);
            self.opaque_index_buffer.push(prev_vert_len + 3);
            self.opaque_index_buffer.push(prev_vert_len + 1);
        } else {
            self.opaque_index_buffer.push(prev_vert_len + 0);
            self.opaque_index_buffer.push(prev_vert_len + 2);
            self.opaque_index_buffer.push(prev_vert_len + 3);
            self.opaque_index_buffer.push(prev_vert_len + 0);
            self.opaque_index_buffer.push(prev_vert_len + 3);
            self.opaque_index_buffer.push(prev_vert_len + 1);
        }

        self
    }

    pub fn quad_transparent(
        &mut self,
        pos: [Vec3; 4],
        lights: [u32; 4],
        color: Vec3,
        normal: Vec3,
        uv_min: Vec2,
        uv_max: Vec2,
        flip_vertex_order: bool,
    ) -> &mut Self {

        let prev_vert_len = self.transparent_vertex_buffer.len() as u32;
        // Top Left
        self.vertex_transparent(TerrainVertex::new(pos[0], color, normal, uv_min, lights[0]));
        // Top Right
        self.vertex_transparent(TerrainVertex::new(
            pos[1],
            color,
            normal,
            Vec2::new(uv_max.x, uv_min.y),
            lights[1],
        ));
        // Bottom Left
        self.vertex_transparent(TerrainVertex::new(
            pos[2],
            color,
            normal,
            Vec2::new(uv_min.x, uv_max.y),
            lights[2],
        ));
        // Bottom Right
        self.vertex_transparent(TerrainVertex::new(pos[3], color, normal, uv_max, lights[3]));

        if flip_vertex_order {
            self.transparent_index_buffer.push(prev_vert_len + 0);
            self.transparent_index_buffer.push(prev_vert_len + 2);
            self.transparent_index_buffer.push(prev_vert_len + 1);
            self.transparent_index_buffer.push(prev_vert_len + 2);
            self.transparent_index_buffer.push(prev_vert_len + 3);
            self.transparent_index_buffer.push(prev_vert_len + 1);
        } else {
            self.transparent_index_buffer.push(prev_vert_len + 0);
            self.transparent_index_buffer.push(prev_vert_len + 2);
            self.transparent_index_buffer.push(prev_vert_len + 3);
            self.transparent_index_buffer.push(prev_vert_len + 0);
            self.transparent_index_buffer.push(prev_vert_len + 3);
            self.transparent_index_buffer.push(prev_vert_len + 1);
        }

        self
    }

    /// Adds a vertex to a buffer, private because it doesn't update the index buffer
    fn vertex(&mut self, vert: TerrainVertex) -> &mut Self {
        self.opaque_vertex_buffer.push(vert);
        self
    }

    fn vertex_transparent(&mut self, vert: TerrainVertex) -> &mut Self {
        self.transparent_vertex_buffer.push(vert);
        self
    }

    /// Constructs a mesh from the buffers of vertices, then empties the buffers
    pub fn build(&mut self, device: &Device) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.opaque_vertex_buffer.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.opaque_index_buffer.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_vertices = self.opaque_vertex_buffer.len() as u32;
        let num_indices = self.opaque_index_buffer.len() as u32;

        let vertex_buffer_transparent = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.transparent_vertex_buffer.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer_transparent = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.transparent_index_buffer.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_vertices_transparent = self.transparent_vertex_buffer.len() as u32;
        let num_indices_transparent = self.transparent_index_buffer.len() as u32;
        let mesh = Mesh::new(vertex_buffer, vertex_buffer_transparent, num_vertices, num_vertices_transparent, index_buffer, index_buffer_transparent, num_indices, num_indices_transparent);

        self.opaque_vertex_buffer.clear();
        self.transparent_vertex_buffer.clear();
        self.opaque_index_buffer.clear();
        self.transparent_index_buffer.clear();

        mesh
    }

    // pub fn into_mesh(&mut self, queue: &Queue, mesh: &mut Mesh) {
    //     let vertex_buffer = &mesh.vertex_buffer;
    //     let index_buffer = &mesh.index_buffer;
    //
    //     queue.write_buffer(
    //         vertex_buffer,
    //         0,
    //         bytemuck::cast_slice(self.vertex_buffer.as_slice()),
    //     );
    //     queue.write_buffer(
    //         index_buffer,
    //         0,
    //         bytemuck::cast_slice(self.index_buffer.as_slice()),
    //     );
    //
    //     mesh.num_verticies = self.vertex_buffer.len() as u32;
    //     mesh.num_indicies = self.index_buffer.len() as u32;
    //
    //     self.vertex_buffer.clear();
    //     self.index_buffer.clear();
    // }

    // TODO: Use states rather than blocks
    fn get_occlusions(nearby_blocks: &[TBlockData; 26], states: &Register<BlockState>, source_block_transparent: bool, source_block_id: usize) -> u32 {
        let mut occlusions = 0u32;
        for dir in &DIRECTIONS {
            let state_id = nearby_blocks[dir.ordinal()] as usize;
            if let Some(state) = states.get_element_from_index(state_id).as_ref() {
                let block = state.get_block();
                let block_id = state.get_block_id();
                let block_transparent = block.is_transparent();

                let both_transparent = block_transparent && block_transparent == source_block_transparent;
                // let different_transparencies = block_transparent != source_block_transparent;
                let same_block = block_id == source_block_id;

                let other_culls_this = block.culls_side(dir.reverse());
                
                let should_occlude = (both_transparent && same_block) || (!block_transparent && other_culls_this);
                occlusions |= (should_occlude as u32) << dir.ordinal();
            }
        }
        occlusions
    }

    fn find_texture_in_map(texture_strings: &HashMap<String, String>, mut tex_to_find: String) -> String {
        let default_texture = String::from("missing");

        while tex_to_find.starts_with("#") {
            let a = &texture_strings.get(&tex_to_find[1..]);
            tex_to_find = a.unwrap_or(&default_texture).clone();
        }

        return tex_to_find;
    }

    //Vec<Option<&ChunkSection>>
    fn get_nearby_blocks(chunk: &Chunk, nearby_chunks: &ChunkStorage<Chunk>, intra_chunk_position: IVec3, chunk_position: IVec3) -> [TBlockData; 26] {
        let mut nearby_blocks = [0; 26];
        for dir in &DIRECTIONS_ALL {
            let dir_index = dir.ordinal();
            let new_pos = intra_chunk_position + dir.get_int_vector();

            // 16 -> 1, -1 -> -1, [0, 15] -> 0
            let cx = new_pos.x >> 4;
            let cy = new_pos.y >> 4;
            let cz = new_pos.z >> 4;

            let state_index =  if cx | cy | cz != 0 {
                if let Ok(chunk) = nearby_chunks.get_chunk(chunk_position + IVec3::new(cx, cy, cz)) {
                    chunk.get_block_at_pos((new_pos.x as u32) & 15, (new_pos.y as u32) & 15, (new_pos.z as u32) & 15)
                } else {
                    continue;
                }
            } else {
                chunk.get_block_at_pos(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32)
            };
            nearby_blocks[dir_index] = state_index;
        }
        nearby_blocks
    }

    fn get_nearby_lights(chunk: &Chunk, nearby_chunks: &ChunkStorage<Chunk>, intra_chunk_position: IVec3, chunk_position: IVec3) -> [(TLightData, TLightData); 26] {
        let mut nearby_blocks = [(0u8, 0u8); 26];
        for dir in &DIRECTIONS_ALL {
            let dir_index = dir.ordinal();
            let new_pos = intra_chunk_position + dir.get_int_vector();

            // 16 -> 1, -1 -> -1, [0, 15] -> 0
            let cx = new_pos.x >> 4;
            let cy = new_pos.y >> 4;
            let cz = new_pos.z >> 4;

            nearby_blocks[dir_index] =  if cx | cy | cz != 0 {
                if let Ok(chunk) = nearby_chunks.get_chunk(chunk_position + IVec3::new(cx, cy, cz)) {
                    chunk.get_light_at_pos((new_pos.x as u32) & 15, (new_pos.y as u32) & 15, (new_pos.z as u32) & 15)
                } else {
                    continue;
                }
            } else {
                chunk.get_light_at_pos(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32)
            };
        }
        nearby_blocks
    }

    fn ao_inside(block: std::rc::Rc<Block>) -> u8 {
        block.is_full_block() as u8
    }

    fn get_ao_for_corner(side1: TBlockData, side2: TBlockData, corner: TBlockData, states: &Register<BlockState> ) -> u8 {
        let state_to_block_fn = |state: std::rc::Rc<BlockState>| { state.get_block() };
        let a = states.get_element_from_index(side1 as usize).map(state_to_block_fn).map(Self::ao_inside).unwrap_or(1u8);
        let b = states.get_element_from_index(side2 as usize).map(state_to_block_fn).map(Self::ao_inside).unwrap_or(1u8);
        let c = states.get_element_from_index(corner as usize).map(state_to_block_fn).map(Self::ao_inside).unwrap_or(1u8);

        if a + b == 2 {
            return 0u8;
        }

        3 - (a + b + c)
    }

    fn get_lights_for_corner(corner: DirectionAll, nearby_blocks: &[(TLightData, TLightData); 26], block_light: u8, sky_light: u8) -> (u8, u8) {
        let dirs = match corner {
            DirectionAll::NED => {
                [ DirectionAll::NED.ordinal(), DirectionAll::NE.ordinal(), DirectionAll::ND.ordinal(), DirectionAll::ED.ordinal(), DirectionAll::North.ordinal(), DirectionAll::East.ordinal(), DirectionAll::Down.ordinal() ]
            },
            DirectionAll::NWD => {
                [ DirectionAll::NWD.ordinal(), DirectionAll::NW.ordinal(), DirectionAll::ND.ordinal(), DirectionAll::WD.ordinal(), DirectionAll::North.ordinal(), DirectionAll::West.ordinal(), DirectionAll::Down.ordinal() ]
            },
            DirectionAll::NEU => {
                [ DirectionAll::NEU.ordinal(), DirectionAll::NE.ordinal(), DirectionAll::NU.ordinal(), DirectionAll::EU.ordinal(), DirectionAll::North.ordinal(), DirectionAll::East.ordinal(), DirectionAll::Up.ordinal() ]
            },
            DirectionAll::NWU => {
                [ DirectionAll::NWU.ordinal(), DirectionAll::NW.ordinal(), DirectionAll::NU.ordinal(), DirectionAll::WU.ordinal(), DirectionAll::North.ordinal(), DirectionAll::West.ordinal(), DirectionAll::Up.ordinal() ]
            },
            DirectionAll::SED => {
                [ DirectionAll::SED.ordinal(), DirectionAll::SE.ordinal(), DirectionAll::SD.ordinal(), DirectionAll::ED.ordinal(), DirectionAll::South.ordinal(), DirectionAll::East.ordinal(), DirectionAll::Down.ordinal() ]
            },
            DirectionAll::SWD => {
                [ DirectionAll::SWD.ordinal(), DirectionAll::SW.ordinal(), DirectionAll::SD.ordinal(), DirectionAll::WD.ordinal(), DirectionAll::South.ordinal(), DirectionAll::West.ordinal(), DirectionAll::Down.ordinal() ]
            },
            DirectionAll::SEU => {
                [ DirectionAll::SEU.ordinal(), DirectionAll::SE.ordinal(), DirectionAll::SU.ordinal(), DirectionAll::EU.ordinal(), DirectionAll::South.ordinal(), DirectionAll::East.ordinal(), DirectionAll::Up.ordinal() ]
            },
            DirectionAll::SWU => {
                [ DirectionAll::SWU.ordinal(), DirectionAll::SW.ordinal(), DirectionAll::SU.ordinal(), DirectionAll::WU.ordinal(), DirectionAll::South.ordinal(), DirectionAll::West.ordinal(), DirectionAll::Up.ordinal() ]
            },
            _ => [0, 0, 0, 0, 0, 0, 0],
        };

        let (a, a1) = nearby_blocks[dirs[0]];
        let (b, b1) = nearby_blocks[dirs[1]];
        let (c, c1) = nearby_blocks[dirs[2]];
        let (d, d1) = nearby_blocks[dirs[3]];
        let (e, e1) = nearby_blocks[dirs[4]];
        let (f, f1) = nearby_blocks[dirs[5]];
        let (g, g1) = nearby_blocks[dirs[6]];

        let mut denom = 0;
        if a > 0 { denom += 1; }
        if b > 0 { denom += 1; }
        if c > 0 { denom += 1; }
        if d > 0 { denom += 1; }
        if e > 0 { denom += 1; }
        if f > 0 { denom += 1; }
        if g > 0 { denom += 1; }
        if block_light > 0 { denom += 1; }
        if denom == 0 { return (0, 0); }
        let block_light = ((a + b + c + d + e + f + g + block_light) / denom) as u8;
        let sky_light = ((a1 + b1 + c1 + d1 + e1 + f1 + g1 + sky_light) / denom) as u8;
        (block_light, sky_light)
    }

    fn get_nearby_lighting_data(nearby_blocks: &[(TLightData, TLightData); 26], block_light: u8, sky_light: u8) -> [(TLightData, TLightData); 8] {
        [
            Self::get_lights_for_corner(DirectionAll::NED, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::NWD, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::NEU, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::NWU, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::SED, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::SWD, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::SEU, nearby_blocks, block_light, sky_light),
            Self::get_lights_for_corner(DirectionAll::SWU, nearby_blocks, block_light, sky_light),
        ]
    }

    fn get_nearby_ao_data(nearby_blocks: &[TBlockData; 26], states: &Register<BlockState>) -> [u8; 24] {
        let mut ao = [3; 24];
        // north
        ao[0] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], states);
        ao[1] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], states);
        ao[2] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], states);
        ao[3] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], states);
        // south
        ao[4] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], states);
        ao[5] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], states);
        ao[6] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], states);
        ao[7] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], states);
        // east
        ao[8] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], states);
        ao[9] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], states);
        ao[10] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], states);
        ao[11] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], states);
        // west
        ao[12] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], states);
        ao[13] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], states);
        ao[14] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], states);
        ao[15] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], states);
        // up
        ao[18] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], states);
        ao[19] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], states);
        ao[16] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], states);
        ao[17] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], states);
        // down
        ao[20] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], states);
        ao[21] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], states);
        ao[22] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], states);
        ao[23] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], states);

        ao
    }

    pub fn trilinear_interpolate(pos: Vec3, samples: &[f32; 8]) -> f32 {
        let xd = pos.x;
        let yd = pos.y;
        let zd = pos.z;

        let c000 = samples[0]; // -1 -1 -1 NED
        let c001 = samples[1]; // -1 -1  1 NWD
        let c010 = samples[2]; // -1  1 -1 NEU
        let c011 = samples[3]; // -1  1  1 NWU
        let c100 = samples[4]; //  1 -1 -1 SED
        let c101 = samples[5]; //  1 -1  1 SWD
        let c110 = samples[6]; //  1  1 -1 SEU
        let c111 = samples[7]; //  1  1  1 SWU

        let xc = 1.0 - xd;
        let c00  = c000 * xc + c100 * xd;
        let c01 = c001 * xc + c101 * xd;
        let c10 = c010 * xc + c110 * xd;
        let c11 = c011 * xc + c111 * xd;

        let yc = 1.0 - yd;
        let c0 = c00 * yc + c10 * yd;
        let c1 = c01 * yc + c11 * yd;

        c0 * (1.0 - zd) + c1 * zd
    }

    pub fn bilinear_interpolate(pos: Vec2, samples: &[f32; 4]) -> f32 {
        let xd = pos.x;
        let yd = pos.y;

        let c00 = samples[0]; // -1 -1 ED
        let c01 = samples[1]; // -1  1 WD
        let c10 = samples[2]; //  1 -1 EU
        let c11 = samples[3]; //  1  1 WU

        let xc = 1.0 - xd;
        let c0 = c00 * xc + c10 * xd;
        let c1 = c01 * xc + c11 * xd;

        c0 * (1.0 - yd) + c1 * yd
    }

    pub fn sample_light_for_pos(pos: Vec3, lights: &[f32; 8], sky_lights: &[f32; 8], ao: u32) -> u32 {
        let a = (Self::trilinear_interpolate(pos, &lights) as u32) & 0b1111;
        // let b = (Self::trilinear_interpolate(pos, &ao) as u32) << 4;
        let b = (ao & 0b1111) << 4;
        let c = (Self::trilinear_interpolate(pos, &sky_lights) as u32) & 0b1111;
        let c = c << 8;
        a | b | c
    }

    pub fn sample_light_for_pos_multiple(pos: &[Vec3; 4], lights: &[(u8, u8); 8], ao: &[u8]) -> [u32; 4] {
        let mut floating_lights = [0.; 8];
        let mut floating_sky_lights = [0.; 8];
        for i in 0..8 {
            let (block_light, sky_light) = lights[i];
            floating_lights[i] = block_light as f32;
            floating_sky_lights[i] = sky_light as f32;
        }
        // let floating_ao = [ao[0] as f32, ao[1] as f32, ao[2] as f32, ao[3] as f32, ao[4] as f32, ao[5] as f32, ao[6] as f32, ao[7] as f32];
        [
            Self::sample_light_for_pos(pos[0], &floating_lights, &floating_sky_lights, ao[0] as u32),
            Self::sample_light_for_pos(pos[1], &floating_lights, &floating_sky_lights, ao[1] as u32),
            Self::sample_light_for_pos(pos[2], &floating_lights, &floating_sky_lights, ao[2] as u32),
            Self::sample_light_for_pos(pos[3], &floating_lights, &floating_sky_lights, ao[3] as u32)
        ]
    }

    // Vec<Option<&ChunkSection>>
    pub fn tessellate_chunk_section(&mut self, section: &Chunk, chunk_real_position: Vec3, chunk_pos: IVec3, blocks: &Register<Block>, states: &Register<BlockState>, models: &HashMap<Identifier, BakedModel>, textures: &HashMap<Identifier, TextureObject>, nearby_chunks: &ChunkStorage<Chunk>) {
        let smooth_shading = true;
        for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                    let real_relative_position = Vec3::new(x as f32, y as f32, z as f32);
                    let real_world_position = chunk_real_position + real_relative_position;
                    let state_id = section.get_block_at_pos(x, y, z);
                    // let metadata = (block_id >> 8) & 0b00001111;
                    // let block_id = (state_id & 0b0000000011111111) as usize;
                    let (sky_light, block_light) = section.get_light_at_pos(x, y, z);
                    // Air, stop
                    if state_id == 0 { continue; }

                    let state = match states.get_element_from_index(state_id.into()) {
                        Some(state) => state,
                        _ => continue,
                    };

                    let block_id = state.get_block_id();
                    let block = state.get_block();
                    let is_transparent = block.is_transparent();

                    let model = match models.get(state.get_state_identifier()) {
                        Some(model) => model,
                        _ => continue,
                    };

                    let intra_chunk_position = IVec3::new(x as i32, y as i32, z as i32);
                    let nearby_blocks = Self::get_nearby_blocks(section, &nearby_chunks, intra_chunk_position, chunk_pos);
                    let nearby_lights = Self::get_nearby_lights(section, &nearby_chunks, intra_chunk_position, chunk_pos);
                    let occlusions = Self::get_occlusions(&nearby_blocks, states, is_transparent, block_id);

                    let lights = Self::get_nearby_lighting_data(&nearby_lights, block_light, sky_light);
                    let ao = if model.ambient_occlusion() { Self::get_nearby_ao_data(&nearby_blocks, states) } else { [3; 24] };

                    let model_textures = model.textures();

                    for face in model.shapes() {
                        match face {
                            ModelShape::Quad {quad} => {
                                if let Some(dir) = quad.cullface {
                                    if (occlusions & dir.ordinal_bitwise()) > 0 {
                                        continue;
                                    }
                                };

                                let positions: [Vec3; 4] = [quad.pos[0] + real_world_position, quad.pos[1] + real_world_position, quad.pos[2] + real_world_position, quad.pos[3] + real_world_position];
                                let texture = Self::find_texture_in_map(model_textures, quad.texture.clone());
                                let texture_id = Identifier::from(texture);
                                let (uv_min, uv_max) = if let Some(TextureObject::AtlasTexture { internal_uv }) = textures.get(&texture_id) {
                                    (internal_uv[0], internal_uv[1])
                                } else { 
                                    log::warn!("No texture for {}", texture_id);
                                    (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)) 
                                };
                                let uv_range = uv_max - uv_min;
                                let (quad_uv_min, quad_uv_max) = (quad.u / 16.0, quad.v / 16.0);

                                let (uv_min, uv_max) = (uv_min + uv_range * quad_uv_min, uv_min + uv_range * quad_uv_max);

                                let ao = if face.ao_face().is_some() {
                                    let ao_range_min = face.ao_face().unwrap().ordinal() * 4;
                                    let ao_range_max = ao_range_min + 4;
                                    &ao[ao_range_min..ao_range_max]
                                } else {
                                    &[3, 3, 3, 3]
                                };

                                let lights = Self::sample_light_for_pos_multiple(&quad.pos, &lights, ao);

                                let ao_left = ao[0] + ao[3];
                                let ao_right = ao[2] + ao[1];
                                let ao_flip = ao_left < ao_right;
                                let light_left = lights[0] + lights[3];
                                let light_right = lights[2] + lights[1];
                                let light_flip = light_left >= light_right;
                                let light_flip = lights[0] >= light_right || lights[3] >= light_right;
                                let flip = light_flip;
                                // let flip = ao_flip;

                                if is_transparent {
                                    self.quad_transparent(positions, lights, quad.color, quad.normal, uv_min, uv_max, flip);
                                } else {
                                    self.quad(positions, lights, quad.color, quad.normal, uv_min, uv_max, flip);
                                }
                            },
                            ModelShape::Triangle {triangle} => {

                            }
                        }
                    }

                } // z
            } // x
        } // y
    }
}
