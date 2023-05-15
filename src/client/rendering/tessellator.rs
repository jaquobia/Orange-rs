use std::collections::HashMap;
use std::ops::Add;

use ultraviolet::{Vec2, Vec3, IVec3};
use wgpu::{util::DeviceExt, Device};

use crate::{direction::DIRECTIONS, world::chunk::{CHUNK_SECTION_AXIS_SIZE, Chunk, ChunkSection}, registry::Register, block::Block};
use crate::client::models::model::{BakedModel, ModelQuad, ModelShape};
use crate::client::textures::TextureObject;
use crate::direction::{DirectionAll, DIRECTIONS_ALL};
use crate::minecraft::identifier::Identifier;

use crate::world::chunk::ChunkDataType;
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

    fn get_occlusions(nearby_blocks: &[ChunkDataType; 26], blocks: &Register<Block>, source_block_transparent: bool) -> u32 {
        let mut occlusions = 0u32;
        for dir in &DIRECTIONS {
            let chunk_data = nearby_blocks[dir.ordinal()];
            let (block_id, _metadata, _block_light) = Chunk::chunk_data_helper(chunk_data);
            if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                let should_occlude = (block.is_transparent() == source_block_transparent) && block.culls_side(dir.reverse());
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
    fn get_nearby_blocks(chunk: &ChunkSection, nearby_chunks: &ChunkStorage<ChunkSection>, intra_chunk_position: IVec3, chunk_position: IVec3) -> [ChunkDataType; 26] {
        let mut nearby_blocks = [0; 26];
        for dir in &DIRECTIONS_ALL {
            let dir_index = dir.ordinal();
            let new_pos = intra_chunk_position + dir.get_int_vector();

            // 16 -> 1, -1 -> -1, [0, 15] -> 0
            let cx = new_pos.x >> 4;
            let cy = new_pos.y >> 4;
            let cz = new_pos.z >> 4;

            nearby_blocks[dir_index] =  if cx | cy | cz != 0 {
                if let Ok(chunk) = nearby_chunks.get_chunk(chunk_position + IVec3::new(cx, cy, cz)) {
                    chunk.get_pos((new_pos.x as u32) & 15, (new_pos.y as u32) & 15, (new_pos.z as u32) & 15)
                } else {
                    continue;
                }
            } else {
                chunk.get_pos(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32)
            };
        }
        nearby_blocks
    }

    fn ao_inside(block: &Block) -> u8 {
        block.is_full_block() as u8
    }

    fn get_ao_for_corner(side1: ChunkDataType, side2: ChunkDataType, corner: ChunkDataType, blocks: &Register<Block> ) -> u8 {
        let a = Chunk::chunk_data_helper(side1);
        let b = Chunk::chunk_data_helper(side2);
        let c = Chunk::chunk_data_helper(corner);

        let a = blocks.get_element_from_index(a.0).map(Self::ao_inside).unwrap_or(1u8);
        let b = blocks.get_element_from_index(b.0).map(Self::ao_inside).unwrap_or(1u8);
        let c = blocks.get_element_from_index(c.0).map(Self::ao_inside).unwrap_or(1u8);

        if a + b == 2 {
            return 0u8;
        }

        3 - (a + b + c)
    }

    fn get_lights_for_corner(corner: DirectionAll, nearby_blocks: &[ChunkDataType; 26], block_light: u16) -> u8 {
        let (a, b, c, d, e, f, g) = match corner {
            DirectionAll::NED => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NED.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NE.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::ND.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::ED.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::North.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::East.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Down.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::NWD => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NWD.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NW.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::ND.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::WD.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::North.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::West.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Down.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::NEU => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NEU.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NE.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NU.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::EU.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::North.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::East.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Up.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::NWU => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NWU.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NW.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::NU.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::WU.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::North.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::West.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Up.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::SED => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SED.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SE.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SD.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::ED.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::South.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::East.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Down.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::SWD => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SWD.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SW.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SD.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::WD.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::South.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::West.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Down.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::SEU => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SEU.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SE.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SU.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::EU.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::South.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::East.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Up.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            DirectionAll::SWU => {
                let (_, _, a) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SWU.ordinal()]);
                let (_, _, b) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SW.ordinal()]);
                let (_, _, c) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::SU.ordinal()]);
                let (_, _, d) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::WU.ordinal()]);
                let (_, _, e) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::South.ordinal()]);
                let (_, _, f) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::West.ordinal()]);
                let (_, _, g) = Chunk::chunk_data_helper(nearby_blocks[DirectionAll::Up.ordinal()]);
                (a, b, c, d, e, f, g)
            },
            _ => (0, 0, 0, 0, 0, 0, 0),
        };

        let mut denom = 0;
        if a > 0 { denom += 1; }
        if b > 0 { denom += 1; }
        if c > 0 { denom += 1; }
        if d > 0 { denom += 1; }
        if e > 0 { denom += 1; }
        if f > 0 { denom += 1; }
        if g > 0 { denom += 1; }
        if block_light > 0 { denom += 1; }
        if denom == 0 { return 0; }
        ((a + b + c + d + e + f + g + block_light) / denom) as u8
    }

    fn get_nearby_lighting_data(nearby_blocks: &[ChunkDataType; 26], block_light: u16) -> [u8; 8] {
        [
            Self::get_lights_for_corner(DirectionAll::NED, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::NWD, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::NEU, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::NWU, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::SED, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::SWD, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::SEU, nearby_blocks, block_light),
            Self::get_lights_for_corner(DirectionAll::SWU, nearby_blocks, block_light),


            // Self::get_lights_for_corner(DirectionAll::SWD, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::SED, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::SWU, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::SEU, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::NWD, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::NED, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::NWU, nearby_blocks, block_light),
            // Self::get_lights_for_corner(DirectionAll::NEU, nearby_blocks, block_light),
        ]
    }

    fn get_nearby_ao_data(nearby_blocks: &[ChunkDataType; 26], blocks: &Register<Block>) -> [u8; 24] {
        let mut ao = [3; 24];
        // north
        ao[0] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], blocks);
        ao[1] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], blocks);
        ao[2] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], blocks);
        ao[3] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], blocks);
        // south
        ao[4] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], blocks);
        ao[5] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], blocks);
        ao[6] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], blocks);
        ao[7] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], blocks);
        // east
        ao[8] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], blocks);
        ao[9] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], blocks);
        ao[10] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::SE.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], blocks);
        ao[11] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::NE.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], blocks);
        // west
        ao[12] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], blocks);
        ao[13] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], blocks);
        ao[14] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::NW.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], blocks);
        ao[15] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::SW.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], blocks);
        // up
        ao[18] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::NEU.ordinal()], blocks);
        ao[19] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::NU.ordinal()], nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::NWU.ordinal()], blocks);
        ao[16] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::EU.ordinal()], nearby_blocks[DirectionAll::SEU.ordinal()], blocks);
        ao[17] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SU.ordinal()], nearby_blocks[DirectionAll::WU.ordinal()], nearby_blocks[DirectionAll::SWU.ordinal()], blocks);
        // down
        ao[20] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::SWD.ordinal()], blocks);
        ao[21] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::SD.ordinal()], nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::SED.ordinal()], blocks);
        ao[22] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::WD.ordinal()], nearby_blocks[DirectionAll::NWD.ordinal()], blocks);
        ao[23] = Self::get_ao_for_corner(nearby_blocks[DirectionAll::ND.ordinal()], nearby_blocks[DirectionAll::ED.ordinal()], nearby_blocks[DirectionAll::NED.ordinal()], blocks);

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

    pub fn sample_light_for_pos(pos: Vec3, lights: &[f32; 8], ao: u32) -> u32 {
        let a = Self::trilinear_interpolate(pos, &lights) as u32;
        // let b = (Self::trilinear_interpolate(pos, &ao) as u32) << 4;
        let b = ao << 4;
        a | b
    }

    pub fn sample_light_for_pos_multiple(pos: &[Vec3; 4], lights: &[u8; 8], ao: &[u8]) -> [u32; 4] {
        let floating_lights = [lights[0] as f32, lights[1] as f32, lights[2] as f32, lights[3] as f32, lights[4] as f32, lights[5] as f32, lights[6] as f32, lights[7] as f32];
        // let floating_ao = [ao[0] as f32, ao[1] as f32, ao[2] as f32, ao[3] as f32, ao[4] as f32, ao[5] as f32, ao[6] as f32, ao[7] as f32];
        [
            Self::sample_light_for_pos(pos[0], &floating_lights, ao[0] as u32),
            Self::sample_light_for_pos(pos[1], &floating_lights, ao[1] as u32),
            Self::sample_light_for_pos(pos[2], &floating_lights, ao[2] as u32),
            Self::sample_light_for_pos(pos[3], &floating_lights, ao[3] as u32)
        ]
    }

    // pub fn sample_light_for_quad(quad: &ModelQuad) -> [u32; 4] {
    //     quad.
    // }

    // Vec<Option<&ChunkSection>>
    pub fn tessellate_chunk_section(&mut self, section: &ChunkSection, chunk_real_position: Vec3, chunk_pos: IVec3, blocks: &Register<Block>, textures: &HashMap<Identifier, TextureObject>, nearby_chunks: &ChunkStorage<ChunkSection>) {
        let smooth_shading = true;
        for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                    let real_relative_position = Vec3::new(x as f32, y as f32, z as f32);
                    let real_world_position = chunk_real_position + real_relative_position;
                    let (block_id, metadata, block_light) = Chunk::chunk_data_helper(section.get_pos(x, y, z));
                    // Air, stop
                    if block_id == 0 { continue; }

                    let block = blocks.get_element_from_index(block_id);
                    let (block_model, is_transparent) = if let Some(block) = block.as_ref() {
                        let model = block.get_model(metadata as u32);
                        let is_transparent = block.is_transparent();
                        (model, is_transparent)
                    } else { (BakedModel::new(), false) };

                    let intra_chunk_position = IVec3::new(x as i32, y as i32, z as i32);
                    let nearby_blocks = Self::get_nearby_blocks(section, &nearby_chunks, intra_chunk_position, chunk_pos);
                    let occlusions = Self::get_occlusions(&nearby_blocks, blocks,is_transparent);

                    let lights = Self::get_nearby_lighting_data(&nearby_blocks, block_light);
                    let ao = if block_model.ambient_occlusion() { Self::get_nearby_ao_data(&nearby_blocks, blocks) } else { [3; 24] };

                    let model_textures = block_model.textures();

                    for face in block_model.shapes() {
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
                                let (uv_min, uv_max) = if let TextureObject::AtlasTexture { internal_uv } = textures.get(&texture_id).expect(format!("No texture for {}", texture_id).as_str()) {
                                    (internal_uv[0], internal_uv[1])
                                } else { (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)) };
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
                                // let ao_flip = ao_left < ao_right;
                                let light_left = lights[0] + lights[3];
                                let light_right = lights[2] + lights[1];
                                let light_flip = light_left >= light_right;
                                let light_flip = lights[0] >= light_right || lights[3] >= light_right;
                                let flip = !light_flip;

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
