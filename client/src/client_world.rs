use std::collections::VecDeque;

use crate::rendering::{mesh::Mesh, tessellator::TerrainTessellator};
use orange_rs::{
    block::Block,
    direction::DIRECTIONS,
    level::{
        chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE, CHUNK_SECTION_AXIS_SIZE_M1},
        dimension::Dimension, World, 
    },
    registry::Register,
};
use ultraviolet::{IVec2, IVec3, UVec3, Vec3};

pub type ClientChunkStorage = Option<Mesh>;

pub struct ClientWorld {
    dimensions: Vec<Dimension>,
    player_level_id: usize,
}

impl World for ClientWorld {

}

impl ClientWorld {

    pub fn new() -> Self {
        Self {
            dimensions: Vec::new(),
            player_level_id: 0,
        }
    }

    pub fn get_dimension(&self, id: usize) -> Option<&Dimension> {
        self.dimensions.get(id)
    }

    pub fn get_dimension_mut(&mut self, id: usize) -> Option<&mut Dimension> {
        self.dimensions.get_mut(id)
    }

    pub fn get_player_level_id(&self) -> usize {
        self.player_level_id
    }

    pub fn get_player_dimension(&self) -> Option<&Dimension> {
        self.get_dimension(self.player_level_id)
    }

    pub fn get_player_dimension_mut(&mut self) -> Option<&mut Dimension> {
        self.get_dimension_mut(self.player_level_id)
    }

    pub fn add_dimension(&mut self, dim: Dimension) {
        self.dimensions.push(dim);
    }

    pub fn tesselate_chunk(
        &mut self,
        // level: &Dimension<ClientChunkStorage>,
        chunk_pos: IVec2,
        tesselator: &mut TerrainTessellator,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        blocks: &Register<Block>,
        ) {
        let level = self.get_player_dimension_mut();
        if level.is_none() {
            return;
        }
        let level = level.unwrap();

        let (chunk_x, chunk_z) = chunk_pos.into();
        let chunk = level.get_chunk_at_mut(chunk_x, chunk_z);

        // If chunk is none, nothing to build
        if chunk.is_none() {
            return;
        }
        let chunk_storage = chunk.unwrap();
        let chunk = &mut chunk_storage.0;
        let meshes = &mut chunk_storage.1;

        let air_id = blocks.get_index_from_identifier("air");
        let mut section_index: usize = 0;
        for section in chunk.get_sections() {
            let section_position = Vec3::new(
                chunk_x as f32,
                (section_index * CHUNK_SECTION_AXIS_SIZE) as f32,
                chunk_z as f32,
                );
            for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                    for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                        let pos_vec = UVec3::new(x, y, z);
                        let pos_ivec = IVec3::new(x as i32, y as i32, z as i32);

                        let position = Vec3::new(x as f32, y as f32, z as f32);
                        let position_extent = position + Vec3::one();
                        let chunk_data = section.get_vec(pos_vec);

                        let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
                        if block_id == air_id {
                            continue;
                        }

                        let block = blocks.get_element_from_index(block_id);
                        let mut occlusions: [bool; 6] = [false; 6];
                        let textures: [u32; 6] = if let Some(block) = block.as_ref() {
                            [block.texture_index() as u32; 6]
                        } else {
                            [0; 6]
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
                                        occlusions[dir_index] = true; // Get information from neighbor
                                                                      // chunk
                                        continue;
                                    }
                            let chunk_data = section.get_vec(UVec3::new(
                                    new_pos.x as u32,
                                    new_pos.y as u32,
                                    new_pos.z as u32,
                                    ));
                            let (block_id, _metadata) = Chunk::chunk_data_helper(chunk_data);
                            if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                                occlusions[dir_index] = block.is_transparent();
                            }
                        }

                        tesselator.cuboid(
                            position + section_position,
                            Vec3::one(),
                            textures,
                            &occlusions,
                            );
                    }
                }
            }

            let wrapped_mesh = meshes.get_mut(section_index).unwrap(); // Should be able to freely
                                                                       // unwrap, its a vector of
                                                                       // options
            if wrapped_mesh.is_none() {
                wrapped_mesh.replace(tesselator.build(device));
            } else {
                let mesh = wrapped_mesh.as_mut().unwrap();
                tesselator.into_mesh(queue, mesh);
            }
            section_index += 1;
        }
    }

    fn tesselate_chunks(
        &mut self,
        tesselator: &mut TerrainTessellator,
        tesselation_queue: &mut VecDeque<IVec2>,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        blocks: &Register<Block>,
        ) {


        for chunk_pos in tesselation_queue {
            self.tesselate_chunk(chunk_pos.clone(), tesselator, queue, device, blocks); 
        }
    }
}
