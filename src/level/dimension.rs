use crate::level::{
    chunk::{Chunk, CHUNK_SECTION_AXIS_SIZE},
    chunk_map::ChunkMap,
    terrain_generator::DefaultTerrainGenerator,
};
use crate::{block::Block, identifier::Identifier, registry::Register};
use ultraviolet::{IVec2, IVec3, UVec2};

pub type DimensionChunkDescriptor = (usize, IVec2);

pub struct Dimension {
    /// Associated name
    pub identifier: Identifier,
    /// Associated id
    pub id: i32,
    /// ECS system for managing entities
    pub entity_world: legion::World,
    pub chunks: ChunkMap<Chunk>,
    /// Represents the number of chunk_sections the world is offset in some direction, useful for
    /// making chunks below y=0
    pub chunk_offset: i32,
    /// Represents how tall the world is in chunks
    pub chunk_height: u32,
    pub tg: DefaultTerrainGenerator,
}

impl Dimension {
    pub fn new(
        identifier: Identifier,
        id: i32,
        chunk_height: u32,
        chunk_offset: i32,
        blocks: &Register<Block>,
    ) -> Self {
        Self {
            identifier,
            id,
            entity_world: legion::World::default(),
            chunks: ChunkMap::new(),
            chunk_offset,
            chunk_height,
            tg: DefaultTerrainGenerator::new(chunk_height, blocks),
        }
    }
    pub fn get_identifier(&self) -> &Identifier {
        &self.identifier
    }
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_chunks(&self) -> &ChunkMap<Chunk> {
        &self.chunks
    }
    pub fn get_chunks_mut(&mut self) -> &mut ChunkMap<Chunk> {
        &mut self.chunks
    }
    pub fn get_chunk_offset(&self) -> i32 {
        self.chunk_offset
    }
    pub fn get_chunk_height(&self) -> u32 {
        self.chunk_height
    }
    pub fn get_terrain_generator(&self) -> &DefaultTerrainGenerator {
        &self.tg
    }

    /// A number used to remove the 1's place by and'ing with the position for feature `large_chunks`
    #[cfg(feature = "large_chunks")]
    const NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER: i32 = !1;

    const CHUNK_POS_SHIFT_AMOUNT: i32 = if cfg!(feature="large_chunks") { 5 } else { 4 };
    const CHUNK_POS_BLOCK_INDEX_TRUNCATE: u32 = if cfg!(feature="large_chunks") { 31 } else { 15 };

    pub fn get_chunk_pos(x: i32, z: i32) -> (i32, i32) {
        #[cfg(feature = "large_chunks")]
        return (
            x >> 4 & Self::NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER,
            z >> 4 & Self::NORMAL_CHUNK_TO_LARGE_CHUNK_MAGIC_NUMBER,
        );
        #[cfg(not(feature = "large_chunks"))]
        return (x >> 4, z >> 4);
    }

    pub fn get_chunk_pos2(x: i32, z: i32) -> (i32, i32) {
        #[cfg(not(feature = "large_chunks"))]
        return (x << 4, z << 4);
        #[cfg(feature = "large_chunks")]
        return (x << 5, z << 5);
    }

    pub fn get_chunk_vec(pos: IVec2) -> IVec2 {
        IVec2::from(Self::get_chunk_pos(pos.x, pos.y))
    }

    fn get_chunk_inner_pos(x: i32, z: i32) -> (u32, u32) {
        #[cfg(feature = "large_chunks")]
        return ((x & 31) as u32, (z & 31) as u32);
        #[cfg(not(feature = "large_chunks"))]
        return ((x & 15) as u32, (z & 15) as u32);
    }

    fn get_chunk_inner_vec(pos: IVec2) -> UVec2 {
        UVec2::from(Self::get_chunk_inner_pos(pos.x, pos.y))
    }

    /// Get the block at an arbitrary point in the world, can be have negative axis and transpire the full extent of
    /// the world
    pub fn get_block_at_pos(&self, x: i32, y: i32, z: i32) -> Option<u64> {
        let chunk_pos = Self::get_chunk_pos(x, z);
        let chunk = self.get_chunks().get_chunk_pos(chunk_pos.0, chunk_pos.1);
        let (x, z) = Self::get_chunk_inner_pos(x, z);
        return if let Some(chunk) = chunk {
            Some(chunk.get_block_at_pos(
                x,
                (y - (self.get_chunk_offset() * CHUNK_SECTION_AXIS_SIZE as i32)) as u32,
                z,
            ))
        } else {
            None
        };
    }

    /// Get the block at an arbitrary point in the world, use a vector instead of a pos
    fn get_block_at_vec(&self, pos: IVec3) -> Option<u64> {
        self.get_block_at_pos(pos.x, pos.y, pos.z)
    }

    pub fn get_chunk_at(&self, x: i32, z: i32) -> Option<&Chunk> {
        let position = IVec2::new(x, z);
        self.get_chunks().get_chunk_vec(position)
    }

    pub fn get_chunk_at_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        let position = IVec2::new(x, z);
        self.get_chunks_mut().get_chunk_vec_mut(position)
    }

    pub fn get_chunk_at_vec(&self, pos: IVec2) -> Option<&Chunk> {
        let position = IVec2::new(pos.x, pos.y);
        self.get_chunks().get_chunk_vec(position)
    }

    pub fn get_chunk_at_vec_mut(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        let position = IVec2::new(pos.x, pos.y);
        self.get_chunks_mut().get_chunk_vec_mut(position)
    }

    pub fn generate_chunk(&mut self, pos: IVec2) {
        let mut chunk = Chunk::new(pos, self.chunk_height as usize);
        self.tg.generate_chunk(&mut chunk);
        self.chunks.set_chunk(pos, Some(chunk));
    }
}
