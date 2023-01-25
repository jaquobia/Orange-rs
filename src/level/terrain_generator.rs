use ultraviolet::{UVec3, UVec2};

use crate::{block::Block, registry::Register};

use super::chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE, ChunkHeightmapType};

pub struct DefaultTerrainGenerator {
    chunk_height: u32,
    grass: usize,
    stone: usize,
    dirt: usize,
    bedrock: usize,
}

impl DefaultTerrainGenerator {
    pub fn new(num_sections: u32, blocks: &Register<Block>) -> Self {
        let chunk_height = num_sections * CHUNK_SECTION_AXIS_SIZE as u32;

        let grass = blocks.get_index_from_identifier("grass");
        let stone = blocks.get_index_from_identifier("stone");
        let dirt = blocks.get_index_from_identifier("dirt");
        let bedrock = blocks.get_index_from_identifier("bedrock");

        Self {
            chunk_height,
            grass,
            stone,
            dirt,
            bedrock,
        }
    }

    fn inner_iter_3d<F: FnMut(UVec3)>(&self, mut f: F) {
        for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for y in 0..self.chunk_height {
                    f(UVec3::new(x, y, z));
                }
            }
        }
    }

    fn inner_iter_2d<F: FnMut(UVec2)>(&self, mut f: F) {
        for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
            for z in 0..CHUNK_SECTION_AXIS_SIZE as u32  {
                f(UVec2::new(x, z));
            }
        }
    }

    pub fn generate_chunk(&self, chunk: &mut Chunk) {
        self.inner_iter_3d(|pos| {
            let y = pos.y;
            let block = if y > 60 {
                0
            } else if y == 60 {
                self.grass
            } else if y >= 56 {
                self.dirt
            } else if y >= 4 {
                self.stone
            } else {
                self.bedrock
            };
            chunk.set_block_at_vec(block as u64, pos)
        });

        self.inner_iter_2d(|pos| {
            let mut heightmap_value: ChunkHeightmapType = (15, 15);
            for y in self.chunk_height as i32 -1..0 {
                if chunk.get_block_at_pos(pos.x, y as u32, pos.y) != 0 {
                    heightmap_value.0 = y;
                    break;
                }
            }
            
        })
    }
}
