use ultraviolet::UVec3;

use super::chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE_M1, CHUNK_SECTION_AXIS_SIZE};


pub struct DefaultTerrainGenerator {
    chunk_height: u32,
}

impl DefaultTerrainGenerator {
    pub fn new(num_sections: u32) -> Self {
        let chunk_height = num_sections * CHUNK_SECTION_AXIS_SIZE as u32;
        Self {
            chunk_height,
        }
    }

    fn inner_iter<F: FnMut(UVec3)>(&self, mut f: F) {
        for x in 0..CHUNK_SECTION_AXIS_SIZE_M1 as u32 {
            for z in 0..CHUNK_SECTION_AXIS_SIZE_M1 as u32 {
                for y in 0..self.chunk_height {
                    f(UVec3::new(x, y, z))
                }
            }
        }
    }

    //TODO: find way to reduce this to an iterator
    pub fn generate_chunk(&self, chunk: &mut Chunk) {
        self.inner_iter(|pos| {
            let y = pos.y;
            let block = if y > 60 {
                0
            } else if y == 60 {
                6
            } else if y >= 56 {
                2
            } else if y >= 4 {
                3
            } else {
                4
            };
            chunk.set_block_at_vec(block, pos) 
        });
    }
}
