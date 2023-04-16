use std::collections::HashMap;
use std::ops::Add;
use log::warn;

use ultraviolet::{Vec2, Vec3, UVec3, IVec3};
use wgpu::{util::DeviceExt, Device, Queue};

use crate::{direction::{Direction, DIRECTIONS}, world::chunk::{CHUNK_SECTION_AXIS_SIZE, CHUNK_SECTION_AXIS_SIZE_M1, Chunk, ChunkSection}, registry::Register, block::Block};
use crate::client::models::model::{BakedModel, ModelShape};
use crate::client::textures::TextureObject;
use crate::minecraft::identifier::Identifier;
use crate::registry::Registerable;

use super::{mesh::Mesh, verticies::TerrainVertex};

pub struct TerrainTessellator {
    opaque_vertex_buffer: Vec<TerrainVertex>,
    transparent_vertex_buffer: Vec<TerrainVertex>,
    opaque_index_buffer: Vec<u32>,
    transparent_index_buffer: Vec<u32>,
}

impl TerrainTessellator {
    /// Construct a new tessellator object
    pub fn new() -> Self {
        Self {
            opaque_vertex_buffer: vec![],
            transparent_vertex_buffer: vec![],
            opaque_index_buffer: vec![],
            transparent_index_buffer: vec![],
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
        color: Vec3,
        normal: Vec3,
        uv_min: Vec2,
        uv_max: Vec2,
    ) -> &mut Self {

        let prev_vert_len = self.opaque_vertex_buffer.len() as u32;
        // Top Left
        self.vertex(TerrainVertex::new(pos[0], color, normal, uv_min, 0, 0));
        // Top Right
        self.vertex(TerrainVertex::new(
            pos[1],
            color,
            normal,
            Vec2::new(uv_max.x, uv_min.y),
            0,
            0,
        ));
        // Bottom Left
        self.vertex(TerrainVertex::new(
            pos[2],
            color,
            normal,
            Vec2::new(uv_min.x, uv_max.y),
            0,
            0,
        ));
        // Bottom Right
        self.vertex(TerrainVertex::new(pos[3], color, normal, uv_max, 0, 0));

        self.opaque_index_buffer.push(prev_vert_len + 0);
        self.opaque_index_buffer.push(prev_vert_len + 2);
        self.opaque_index_buffer.push(prev_vert_len + 3);
        self.opaque_index_buffer.push(prev_vert_len + 0);
        self.opaque_index_buffer.push(prev_vert_len + 3);
        self.opaque_index_buffer.push(prev_vert_len + 1);

        self
    }

    pub fn quad_transparent(
        &mut self,
        pos: [Vec3; 4],
        color: Vec3,
        normal: Vec3,
        uv_min: Vec2,
        uv_max: Vec2,
    ) -> &mut Self {

        let prev_vert_len = self.transparent_vertex_buffer.len() as u32;
        // Top Left
        self.vertex_transparent(TerrainVertex::new(pos[0], color, normal, uv_min, 0, 0));
        // Top Right
        self.vertex_transparent(TerrainVertex::new(
            pos[1],
            color,
            normal,
            Vec2::new(uv_max.x, uv_min.y),
            0,
            0,
        ));
        // Bottom Left
        self.vertex_transparent(TerrainVertex::new(
            pos[2],
            color,
            normal,
            Vec2::new(uv_min.x, uv_max.y),
            0,
            0,
        ));
        // Bottom Right
        self.vertex_transparent(TerrainVertex::new(pos[3], color, normal, uv_max, 0, 0));

        self.transparent_index_buffer.push(prev_vert_len + 0);
        self.transparent_index_buffer.push(prev_vert_len + 2);
        self.transparent_index_buffer.push(prev_vert_len + 3);
        self.transparent_index_buffer.push(prev_vert_len + 0);
        self.transparent_index_buffer.push(prev_vert_len + 3);
        self.transparent_index_buffer.push(prev_vert_len + 1);

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

    fn get_occlusions(intra_chunk_position: IVec3, blocks: &Register<Block>, chunk: &ChunkSection, nearby_chunks: &Vec<Option<&ChunkSection>>, source_block_transparent: bool) -> [bool; 6] {
        let mut occlusions: [bool; 6] = [false; 6];
        for dir in &DIRECTIONS {
            let dir_index = dir.ordinal();
            let new_pos = intra_chunk_position + dir.get_int_vector();
            if new_pos.x < 0
                || new_pos.x > CHUNK_SECTION_AXIS_SIZE_M1 as i32
                || new_pos.y < 0
                || new_pos.y > CHUNK_SECTION_AXIS_SIZE_M1 as i32
                || new_pos.z < 0
                || new_pos.z > CHUNK_SECTION_AXIS_SIZE_M1 as i32
            {
                if let Some(chunk) = nearby_chunks[dir_index] {
                    let chunk_data = chunk.get_pos((new_pos.x as u32) & 15, (new_pos.y as u32) & 15, (new_pos.z as u32) & 15);
                    let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
                    if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                        occlusions[dir_index] = (block.is_transparent() == source_block_transparent) && block.culls_side(dir.reverse());
                    }
                }
                continue;
            }
            let chunk_data = chunk.get_pos(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32);
            let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
            if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                occlusions[dir_index] = (block.is_transparent() == source_block_transparent) && block.culls_side(dir.reverse());
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

    pub fn tessellate_chunk_section(&mut self, section: &ChunkSection, section_position: Vec3, blocks: &Register<Block>, textures: &HashMap<Identifier, TextureObject>, nearby_chunks: Vec<Option<&ChunkSection>>) {
        for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {

                    let real_world_position = section_position + Vec3::new(x as f32, y as f32, z as f32);
                    let chunk_data = section.get_vec(UVec3::new(x, y, z));
                    let (block_id, metadata) = Chunk::chunk_data_helper(chunk_data);
                    // Air, stop
                    if block_id == 0 { continue; }

                    let block = blocks.get_element_from_index(block_id);
                    let (block_model, is_transparent) = if let Some(block) = block.as_ref() {
                        let model = block.get_model(metadata as u32);
                        let is_transparent = block.is_transparent();
                        (model, is_transparent)
                    } else { (BakedModel::new(), false) };
                    let occlusions: [bool; 6] = Self::get_occlusions(IVec3::new(x as i32, y as i32, z as i32), blocks, section, &nearby_chunks, is_transparent);
                    let model_textures = block_model.textures();
                    for face in block_model.shapes() {
                        match face {
                            ModelShape::Quad {quad} => {
                                if let Some(dir) = quad.cullface {
                                    if occlusions[dir.ordinal()] {
                                        continue;
                                    }
                                };

                                let positions: [Vec3; 4] = [quad.pos[0] + real_world_position, quad.pos[1] + real_world_position, quad.pos[2] + real_world_position, quad.pos[3] + real_world_position];
                                let qtexclone = quad.texture.clone();
                                let texture = Self::find_texture_in_map(model_textures, qtexclone.clone());
                                let texture_id = Identifier::from(texture);
                                let (uv_min, uv_max) = if let TextureObject::AtlasTexture { internal_uv } = textures.get(&texture_id).expect(format!("No texture for {}", texture_id).as_str()) {
                                    (internal_uv[0], internal_uv[1])
                                } else { (Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)) };
                                let uv_range = uv_max - uv_min;
                                let (quad_uv_min, quad_uv_max) = (quad.u / 16.0, quad.v / 16.0);

                                let (uv_min, uv_max) = (uv_min + uv_range * quad_uv_min, uv_min + uv_range * quad_uv_max);

                                if is_transparent {
                                    self.quad_transparent(positions, quad.color, quad.normal, uv_min, uv_max);
                                } else {
                                    self.quad(positions, quad.color, quad.normal, uv_min, uv_max);
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
