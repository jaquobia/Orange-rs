use std::ops::Add;

use ultraviolet::{Vec2, Vec3, UVec3, IVec3};
use wgpu::{util::DeviceExt, Device, Queue};

use crate::{direction::{Direction, DIRECTIONS}, world::chunk::{CHUNK_SECTION_AXIS_SIZE, CHUNK_SECTION_AXIS_SIZE_M1, Chunk, ChunkSection}, registry::Register, block::Block};

use super::{mesh::Mesh, verticies::TerrainVertex};

pub struct TerrainTessellator {
    vertex_buffer: Vec<TerrainVertex>,
    index_buffer: Vec<u32>,
}

impl TerrainTessellator {
    /// Construct a new tessellator object
    pub fn new() -> Self {
        Self {
            vertex_buffer: vec![],
            index_buffer: vec![],
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
            self.quad(pos, pos_max, color, dir, uv_min, uv_max, lights);
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
        pos_min: Vec3,
        pos_max: Vec3,
        color: Vec3,
        direction: &Direction,
        uv_min: Vec2,
        uv_max: Vec2,
        _lights: &[u32; 8],
    ) -> &mut Self {
        let (p0, p1, p2, p3) = match direction {
            Direction::North => (
                Vec3::new(pos_min.x, pos_max.y, pos_min.z),
                Vec3::new(pos_min.x, pos_max.y, pos_max.z),
                Vec3::new(pos_min.x, pos_min.y, pos_min.z),
                Vec3::new(pos_min.x, pos_min.y, pos_max.z),
            ),
            Direction::South => (
                Vec3::new(pos_max.x, pos_max.y, pos_max.z),
                Vec3::new(pos_max.x, pos_max.y, pos_min.z),
                Vec3::new(pos_max.x, pos_min.y, pos_max.z),
                Vec3::new(pos_max.x, pos_min.y, pos_min.z),
            ),
            Direction::East => (
                Vec3::new(pos_max.x, pos_max.y, pos_min.z),
                Vec3::new(pos_min.x, pos_max.y, pos_min.z),
                Vec3::new(pos_max.x, pos_min.y, pos_min.z),
                Vec3::new(pos_min.x, pos_min.y, pos_min.z),
            ),
            Direction::West => (
                Vec3::new(pos_min.x, pos_max.y, pos_max.z),
                Vec3::new(pos_max.x, pos_max.y, pos_max.z),
                Vec3::new(pos_min.x, pos_min.y, pos_max.z),
                Vec3::new(pos_max.x, pos_min.y, pos_max.z),
            ),
            Direction::Up => (
                Vec3::new(pos_max.x, pos_max.y, pos_min.z),
                Vec3::new(pos_max.x, pos_max.y, pos_max.z),
                Vec3::new(pos_min.x, pos_max.y, pos_min.z),
                Vec3::new(pos_min.x, pos_max.y, pos_max.z),
            ),
            Direction::Down => (
                Vec3::new(pos_max.x, pos_min.y, pos_max.z),
                Vec3::new(pos_max.x, pos_min.y, pos_min.z),
                Vec3::new(pos_min.x, pos_min.y, pos_max.z),
                Vec3::new(pos_min.x, pos_min.y, pos_min.z),
            ),
        };

        let normal = direction.get_float_vector();

        let prev_vert_len = self.vertex_buffer.len() as u32;
        // Top Left
        self.vertex(TerrainVertex::new(p0, color, normal, uv_min, 0, 0));
        // Top Right
        self.vertex(TerrainVertex::new(
            p1,
            color,
            normal,
            Vec2::new(uv_max.x, uv_min.y),
            0,
            0,
        ));
        // Bottom Left
        self.vertex(TerrainVertex::new(
            p2,
            color,
            normal,
            Vec2::new(uv_min.x, uv_max.y),
            0,
            0,
        ));
        // Bottom Right
        self.vertex(TerrainVertex::new(p3, color, normal, uv_max, 0, 0));

        self.index_buffer.push(prev_vert_len + 0);
        self.index_buffer.push(prev_vert_len + 2);
        self.index_buffer.push(prev_vert_len + 3);
        self.index_buffer.push(prev_vert_len + 0);
        self.index_buffer.push(prev_vert_len + 3);
        self.index_buffer.push(prev_vert_len + 1);

        self
    }

    /// Adds a vertex to a buffer, private because it doesn't update the index buffer
    fn vertex(&mut self, vert: TerrainVertex) -> &mut Self {
        self.vertex_buffer.push(vert);
        self
    }

    /// Constructs a mesh from the buffers of vertices, then empties the buffers
    pub fn build(&mut self, device: &Device) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.vertex_buffer.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(self.index_buffer.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_verticies = self.vertex_buffer.len() as u32;
        let num_indicies = self.index_buffer.len() as u32;
        let mesh = Mesh::new(vertex_buffer, num_verticies, index_buffer, num_indicies);

        self.vertex_buffer.clear();
        self.index_buffer.clear();
        // TODO check if keeping buffer size is good
        // self.vertex_buffer.shrink_to_fit();
        // self.index_buffer.shrink_to_fit();

        mesh
    }

    pub fn into_mesh(&mut self, queue: &Queue, mesh: &mut Mesh) {
        let vertex_buffer = &mesh.vertex_buffer;
        let index_buffer = &mesh.index_buffer;

        queue.write_buffer(
            vertex_buffer,
            0,
            bytemuck::cast_slice(self.vertex_buffer.as_slice()),
        );
        queue.write_buffer(
            index_buffer,
            0,
            bytemuck::cast_slice(self.index_buffer.as_slice()),
        );

        mesh.num_verticies = self.vertex_buffer.len() as u32;
        mesh.num_indicies = self.index_buffer.len() as u32;

        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }


    pub fn tessellate_chunk_section(&mut self, section: &ChunkSection, section_position: Vec3, blocks: &Register<Block>, nearby_chunks: Vec<Option<&ChunkSection>>) {

        for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                    let pos_vec = UVec3::new(x, y, z);
                    let pos_ivec = IVec3::new(x as i32, y as i32, z as i32);

                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    let chunk_data = section.get_vec(pos_vec);

                    let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
                    if block_id == 0 {
                        // Air, stop
                        continue;
                    }

                    let block = blocks.get_element_from_index(block_id);
                    let mut occlusions: [bool; 6] = [false; 6];
                    let textures: [u32; 6] = if let Some(block) = block.as_ref() {
                        [block.texture_index() as u32; 6]
                    } else {
                        [10; 6]
                    };

                    for dir in &DIRECTIONS {
                        let dir_index = dir.ordinal();
                        let new_pos = pos_ivec + dir.get_int_vector();
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
                                    occlusions[dir_index] = block.is_transparent();
                                }
                            } else {
                                occlusions[dir_index] = true;
                            };
                            continue;
                        }
                        let chunk_data = section.get_pos(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32, );
                        let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
                        if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                            occlusions[dir_index] = block.is_transparent();
                        }
                    }

                    self.cuboid(
                        position + section_position,
                        Vec3::one(),
                        textures,
                        &occlusions,
                    );
                }
            }
        }
    }
}
