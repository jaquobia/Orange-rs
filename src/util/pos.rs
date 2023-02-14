use ultraviolet::{UVec3, IVec3, Vec3, IVec2};

use crate::world::chunk::CHUNK_SECTION_AXIS_SIZE;

pub trait Position {
    fn to_entity_pos(&self) -> EntityPos;
    fn to_block_pos(&self) -> BlockPos;
    fn to_chunk_pos(&self) -> ChunkPos;
    fn to_inner_chunk_pos(&self) -> InnerChunkPos;
}

pub type EntityPos = Vec3;
pub type BlockPos = IVec3;
pub type ChunkPos = IVec2;
pub type InnerChunkPos = (UVec3, u32);

impl Position for EntityPos {
    fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x as i32, self.y as i32, self.z as i32)
    }
    fn to_chunk_pos(&self) -> ChunkPos {
        ChunkPos::new(self.x as i32 >> 4, self.z as i32 >> 4)
    }
    fn to_inner_chunk_pos(&self) -> InnerChunkPos {
        let uy = self.y as u32;
        let section = uy / CHUNK_SECTION_AXIS_SIZE as u32;
        let height = uy % CHUNK_SECTION_AXIS_SIZE as u32;
        (UVec3::new(self.x as u32 & 15, height, self.z as u32 & 15), section)
    }
    fn to_entity_pos(&self) -> EntityPos {
        self.clone()
    }
}

impl Position for BlockPos {
    fn to_block_pos(&self) -> BlockPos {
        self.clone()
    }
    fn to_chunk_pos(&self) -> ChunkPos {
        ChunkPos::new(self.x >> 4, self.z >> 4)
    }
    fn to_inner_chunk_pos(&self) -> InnerChunkPos {
        let uy = self.y as u32;
        let section = uy / CHUNK_SECTION_AXIS_SIZE as u32;
        let height = uy % CHUNK_SECTION_AXIS_SIZE as u32;
        (UVec3::new(self.x as u32 & 15, height, self.z as u32 & 15), section)
    }
    fn to_entity_pos(&self) -> EntityPos {
        EntityPos::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl Position for ChunkPos {

    /// Intended to be summed with a chunk_pos_inner.to_block_pos()
    fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x << 4, 0, self.y << 4)
    }

    fn to_chunk_pos(&self) -> ChunkPos {
        self.clone()
    }

    fn to_inner_chunk_pos(&self) -> InnerChunkPos {
        self.to_block_pos().to_inner_chunk_pos()
    }

    fn to_entity_pos(&self) -> EntityPos {
        self.to_block_pos().to_entity_pos()
    }
}

pub trait InnerChunkPosTrait {
    fn x(&self) -> u32;
    fn y(&self) -> u32;
    fn z(&self) -> u32;
    fn section(&self) -> u32;

    fn from_full_y(x: u32, y: u32, z: u32) -> InnerChunkPos;
}

impl InnerChunkPosTrait for InnerChunkPos {
    fn x(&self) -> u32 {
        self.0.x
    }
    fn y(&self) -> u32 {
        self.0.y
    }
    fn z(&self) -> u32 {
        self.0.z
    }
    fn section(&self) -> u32 {
        self.1
    }
    fn from_full_y(x: u32, y: u32, z: u32) -> InnerChunkPos {
        (UVec3::new(x, y % CHUNK_SECTION_AXIS_SIZE as u32, z), y / CHUNK_SECTION_AXIS_SIZE as u32)
    }
}

impl Position for InnerChunkPos {
    /// Intended to be summed with a chunk_pos.to_block_pos()
    fn to_block_pos(&self) -> BlockPos {
        BlockPos::new(self.0.x as i32, (self.0.y + (self.1 * CHUNK_SECTION_AXIS_SIZE as u32)) as i32, self.0.z as i32)
    }

    /// There is not enough information to turn an InnerChunkPos into a ChunkPos
    fn to_chunk_pos(&self) -> ChunkPos {
        ChunkPos::zero()
    }

    fn to_inner_chunk_pos(&self) -> InnerChunkPos {
        self.clone()
    }

    fn to_entity_pos(&self) -> EntityPos {
        self.to_block_pos().to_entity_pos()
    }
}
