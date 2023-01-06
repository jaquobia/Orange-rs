use legion::World;
use ultraviolet::{IVec3, IVec2, UVec2};

use crate::{identifier::Identifier, rendering::tessellator::TerrainTessellator, block::Block, registry::Register};

use self::{chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE}, terrain_generator::DefaultTerrainGenerator};

pub mod chunk;
pub mod terrain_generator;

pub struct Level {
    /// Associated name
    identifier: Identifier,
    /// Associated id
    id: i32,
    /// ECS system for managing entities
    entity_world: World,
    chunks: ChunkMap,
    /// Represents the number of chunk_sections the world is offset in some direction, useful for
    /// making chunks below y=0
    chunk_offset: i32,
    /// Represents how tall the world is in chunks
    chunk_height: u32,
    tg: DefaultTerrainGenerator,
}

impl Level {
    pub fn new(identifier: Identifier, id: i32, chunk_height: u32, chunk_offset: i32) -> Self {
        Self {
            identifier,
            id,
            entity_world: World::default(),
            chunks: ChunkMap::new(),
            chunk_offset,
            chunk_height,
            tg: DefaultTerrainGenerator::new(chunk_height),
        }
    }

    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }

    /// A number used to remove the 1's place by and'ing with the position for feature `large_chunks`
    #[cfg(feature = "large_chunks")]
    const NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER: i32 = !1;

    pub fn get_chunk_pos(x: i32, z: i32) -> (i32, i32) {
        #[cfg(feature = "large_chunks")]
        return (x  >> 4 & NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER, z >> 4 & NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER);
        #[cfg(not(feature = "large_chunks"))]
        return (x  >> 4, z >> 4);
    }

    pub fn get_chunk_vec(pos: IVec2) -> IVec2 {
        IVec2::from(Self::get_chunk_pos(pos.x, pos.y))
    }

    pub fn get_chunk_inner_pos(x: i32, z: i32) -> (u32, u32) {
        #[cfg(feature = "large_chunks")]
        return ((x & 31) as u32, (z & 31) as u32);
        #[cfg(not(feature = "large_chunks"))]
        return ((x & 15) as u32, (z & 15) as u32);
    }

    pub fn get_chunk_inner_vec(pos: IVec2) -> UVec2 {
       UVec2::from(Self::get_chunk_inner_pos(pos.x, pos.y)) 
    }

    /// Get the block at an arbitrary point in the world, can be have negative axis and transpire the full extent of
    /// the world
    pub fn get_block_at_pos(&self, x: i32, y: i32, z: i32) -> Option<u64> {
        let chunk_pos = Self::get_chunk_pos(x, z);
        let chunk = self.chunks.get_chunk_pos(chunk_pos.0, chunk_pos.1);
        let (x, z) = Self::get_chunk_inner_pos(x, z);
        return if let Some(chunk) = chunk {
            Some(chunk.get_block_at_pos(x, (y - (self.chunk_offset * CHUNK_SECTION_AXIS_SIZE as i32)) as u32, z))
        } else {
            None
        };
    }

    /// Get the block at an arbitrary point in the world, use a vector instead of a pos
    pub fn get_block_at_vec(&self, pos: IVec3) -> Option<u64> {
        self.get_block_at_pos(pos.x, pos.y, pos.z)
    }

    pub fn get_chunk_at(&self, x: i32, z: i32) -> Option<&Chunk> {
        let position = IVec2::new(x, z);
        self.chunks.get_chunk_vec(position).as_ref()
    }

    pub fn get_chunk_at_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        let position = IVec2::new(x, z);
        self.chunks.get_chunk_vec_mut(position).as_mut()
    }

    pub fn generate_chunks(&mut self) {
        for i in 0..8 {
            for k in 0..8 {
                let position = IVec2::new(i-4, k-4);

                if self.chunks.get_chunk_vec(position).is_none() { 
                    let mut chunk = Chunk::new(position, self.chunk_height as usize);
                    self.tg.generate_chunk(&mut chunk);
                    self.chunks.set_chunk(position, chunk);
                }
            }
        }
    }

    pub fn tesselate_chunks(&mut self, tesselator: &mut TerrainTessellator, queue: &wgpu::Queue, device: &wgpu::Device, blocks: &Register<Block>) {
        for i in -4..4 {
            for j in -4..4 {
                println!("Tessellating chunk: {} {}", i, j);
                let chunk = self.get_chunk_at_mut(i, j);
                if let Some(chunk) = chunk {
                    println!("Chunk Existed");
                    chunk.tesselate(tesselator, queue, device, blocks);
                } 
            }
        }
    }
}


/// Dummy Chunk Storage that has an 9x9 map of chunks
/// Dimensions from -3 -> 
struct ChunkMap {
    chunks: Vec<Vec<Option<Chunk>>>,
}

impl ChunkMap {

    pub fn new() -> Self {
        let mut chunks = Vec::with_capacity(9);
        for i in 0..8 {
            chunks.push(vec![]);
            for _ in 0..8 {
                chunks[i].push(None);
            }
        }
        
        Self {
            chunks,
        }
    }

    pub fn get_chunk_pos(&self, x: i32, z: i32) -> &Option<Chunk> {
        let (x, z) = ((x + 4) as usize, (z + 4) as usize);
        &self.chunks[x][z] 
    }

    pub fn get_chunk_pos_mut(&mut self, x: i32, z: i32) -> &mut Option<Chunk> {
        let (x, z) = ((x + 4) as usize, (z + 4) as usize);
        &mut self.chunks[x][z]
    }

    pub fn get_chunk_vec(&self, pos: IVec2) -> &Option<Chunk> {
        self.get_chunk_pos(pos.x, pos.y)
    }

    pub fn get_chunk_vec_mut(&mut self, pos: IVec2) -> &mut Option<Chunk> {
        self.get_chunk_pos_mut(pos.x, pos.y)
    }

    pub fn set_chunk(&mut self, pos: IVec2, chunk: Chunk) {
        let (x, z) = ((pos.x + 4) as usize, (pos.y + 4) as usize);
        self.chunks[x][z] = Some(chunk);
    }
}
