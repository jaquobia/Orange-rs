use std::collections::HashMap;
use ultraviolet::IVec3;
use crate::direction::DIRECTIONS;
pub mod chunk;

#[derive(Debug)]
pub enum ChunkAccessError {
    PositionOutOfBounds,
    ChunkDoesNotExist,
}

pub type ChunkAccessResult<T> = Result<T, ChunkAccessError>;

pub struct ChunkStack<ChunkType> {
    pub chunks: Vec<Option<ChunkType>>
}

impl<ChunkType> ChunkStack<ChunkType> {
    pub fn new(height: usize) -> Self {
        let mut chunks = Vec::with_capacity(height);
        for x in 0..height {
            chunks.push(None);
        }
        Self {
            chunks,
        }
    }
}

/**
Acts as an abstract wrapper for specified chunk containers, container types included are:
 Planar, Planar Limited, and Cubic
*/
pub enum ChunkStorage<ChunkType> {
    /**
    Chunks are stored on an (almost) infinite 2D grid within a specified number of chunks per 2D position
    works well if the number of chunks in height is small, more chunk height = more storage required per 2D position
    */
    Planar(ChunkStoragePlanar<ChunkType>),
    /**
    Chunks are stored in a 2D grid of limited size and height
    */
    PlanarLimited(ChunkStoragePlanarLimited<ChunkType>),
    /**
    Chunks are stored in a 3D grid
    */
    Cubic(ChunkStorageCubic<ChunkType>),
}

impl<ChunkType> ChunkStorageTrait<ChunkType> for ChunkStorage<ChunkType> {

    fn set_chunk(&mut self, chunk: ChunkType, position: IVec3) -> ChunkAccessResult<()> {
        match self {
            Self::Planar(planar) => { planar.set_chunk(chunk, position) },
            Self::PlanarLimited(planar_limited) => { planar_limited.set_chunk(chunk, position) },
            Self::Cubic(cubic) => { cubic.set_chunk(chunk, position) },
        }
    }

    fn get_chunk(&self, position: IVec3) -> ChunkAccessResult<&ChunkType> {
        match self {
            Self::Planar(planar) => { planar.get_chunk(position) },
            Self::PlanarLimited(planar_limited) => { planar_limited.get_chunk(position) },
            Self::Cubic(cubic) => { cubic.get_chunk(position) },
        }
    }

    fn get_chunk_mut(&mut self, position: IVec3) -> ChunkAccessResult<&mut ChunkType> {
        match self {
            Self::Planar(planar) => { planar.get_chunk_mut(position) },
            Self::PlanarLimited(planar_limited) => { planar_limited.get_chunk_mut(position) },
            Self::Cubic(cubic) => { cubic.get_chunk_mut(position) },
        }
    }

    fn get_or_create_chunk<F: FnOnce() -> ChunkType>(&mut self, position: IVec3, f: F) -> ChunkAccessResult<&mut ChunkType> {
        match self {
            Self::Planar(planar) => { planar.get_or_create_chunk(position, f) },
            Self::PlanarLimited(planar_limited) => { planar_limited.get_or_create_chunk(position, f) },
            Self::Cubic(cubic) => { cubic.get_or_create_chunk(position, f) },
        }
    }

    fn remove_chunk(&mut self, position: IVec3) -> ChunkAccessResult<Option<ChunkType>> {
        match self {
            Self::Planar(planar) => { planar.remove_chunk(position) },
            Self::PlanarLimited(planar_limited) => { planar_limited.remove_chunk(position) },
            Self::Cubic(cubic) => { cubic.remove_chunk(position) },
        }
    }

    fn get_nearby_chunks(&self, position: IVec3) -> Vec<Option<&ChunkType>> {
        match self {
            Self::Planar(planar) => { planar.get_nearby_chunks(position) },
            Self::PlanarLimited(planar_limited) => { planar_limited.get_nearby_chunks(position) },
            Self::Cubic(cubic) => { cubic.get_nearby_chunks(position) },
        }
    }
}

pub trait ChunkStorageTrait<ChunkType> {
    /**
    Emplace the chunk in the storage at position
     */
    fn set_chunk(&mut self, chunk: ChunkType, position: IVec3) -> ChunkAccessResult<()>;
    /**
    Get the chunk at position
    Returns Some(chunk) if a chunk existed, or None if the chunk did not exist or the position was out of bounds
     */
    fn get_chunk(&self, position: IVec3) -> ChunkAccessResult<&ChunkType>;
    /**
    Get the chunk as mutable at position
    Returns Some(chunk) if a chunk existed, or None if the chunk did not exist or the position was out of bounds
     */
    fn get_chunk_mut(&mut self, position: IVec3) -> ChunkAccessResult<&mut ChunkType>;
    /**
    Get the chunk at position if it exists, or creates a new one
    Returns Some(chunk) if a chunk existed or one was created, or None the position was out of bounds
     */
    fn get_or_create_chunk<F: FnOnce() -> ChunkType>(&mut self, position: IVec3, f: F) -> ChunkAccessResult<&mut ChunkType>;
    /**
    Remove the chunk at position
    Returns Some(chunk) if a chunk existed and was removed, or None if the chunk did not exist or the position was out of bounds
     */
    fn remove_chunk(&mut self, position: IVec3) -> ChunkAccessResult<Option<ChunkType>>;

    /**
    Get the optional chunks surrounding the chunk at position, does not guarantee that all chunks exist
     */
    fn get_nearby_chunks(&self, position: IVec3) -> Vec<Option<&ChunkType>>;
}

pub struct ChunkStoragePlanar<ChunkType> {
    height: usize,
    chunk_stacks: Vec<ChunkStack<ChunkType>>,
    stack_pos_to_index_map: HashMap<i64, usize>,
}

impl<ChunkType> ChunkStoragePlanar<ChunkType> {
    pub fn new(height: usize) -> Self {
        const CHUNK_STACK_ALLOC_MAGIC_NUMBER: usize = 21 * 21;
        let chunks = Vec::with_capacity(CHUNK_STACK_ALLOC_MAGIC_NUMBER);
        Self {
            height,
            chunk_stacks: chunks,
            stack_pos_to_index_map: HashMap::new(),
        }
    }

    fn generate_hash(x: i32, z: i32) -> i64 {
        let x: u32 = bytemuck::cast(x);
        let z: u32 = bytemuck::cast(z);
        let x = x as i64;
        let z = z as i64;
        return x | (z << 32); // easiest hash ever, since an i64 is just two i32's
    }

    fn hash_to_index(&self, hash: i64) -> Option<usize> {
        self.stack_pos_to_index_map
            .get(&hash)
            .and_then(|&index| Some(index)) // This line removes the reference
    }

    fn inner_pos_to_index(&self, x: i32, z: i32) -> Option<usize> {
        self.hash_to_index(Self::generate_hash(x, z))
    }

    fn get_check_position(position: IVec3, height: usize) -> ChunkAccessResult<(i32, usize, i32)> {
        let (x, y, z) = position.into();
        if y < 0 || y >= height as i32 {
            return Err(ChunkAccessError::PositionOutOfBounds);
        }
        Ok((x, y as usize, z))
    }

    fn create_stack(&mut self, hash: i64) -> usize {
        let stack_index = self.chunk_stacks.len();
        self.chunk_stacks.push(ChunkStack::new(self.height));
        self.stack_pos_to_index_map.insert(hash, stack_index);
        stack_index
    }

    fn get_stack(&self, x: i32, z: i32) -> Option<&ChunkStack<ChunkType>> {
        let hash = Self::generate_hash(x, z);
        self.hash_to_index(hash).and_then(|index| { Some(&self.chunk_stacks[index]) })
    }
}

impl<ChunkType> ChunkStorageTrait<ChunkType> for ChunkStoragePlanar<ChunkType> {
    fn set_chunk(&mut self, chunk: ChunkType, position: IVec3) -> ChunkAccessResult<()> {
        let (x, y, z) = Self::get_check_position(position, self.height)?;
        let hash = Self::generate_hash(x, z);

        let stack_index = self.hash_to_index(hash).unwrap_or_else(|| {
            // There is no index, therefore no chunk stack, so add one
            self.create_stack(hash)
        });

        self.chunk_stacks[stack_index].chunks[y] = Some(chunk);

        return Ok(());
    }

    fn get_chunk(&self, position: IVec3) -> ChunkAccessResult<&ChunkType> {
        let (x, y, z) = Self::get_check_position(position, self.height)?;
        let hash = Self::generate_hash(x, z);

        self.hash_to_index(hash).and_then(|stack_index| {
            self.chunk_stacks[stack_index].chunks[y].as_ref()
        }).ok_or(ChunkAccessError::ChunkDoesNotExist)
    }

    fn get_chunk_mut(&mut self, position: IVec3) -> ChunkAccessResult<&mut ChunkType> {
        let (x, y, z) = Self::get_check_position(position, self.height)?;
        let hash = Self::generate_hash(x, z);

        self.hash_to_index(hash).and_then(|stack_index| {
            self.chunk_stacks[stack_index].chunks[y].as_mut()
        }).ok_or(ChunkAccessError::ChunkDoesNotExist)
    }

    fn get_or_create_chunk<F: FnOnce() -> ChunkType>(&mut self, position: IVec3, f: F) -> ChunkAccessResult<&mut ChunkType> {
        let (x, y, z) = Self::get_check_position(position, self.height)?;
        let hash = Self::generate_hash(x, z);

        let stack_index = self.hash_to_index(hash).unwrap_or_else(|| {
            self.create_stack(hash)
        });
        let stack = &mut self.chunk_stacks[stack_index];
        if stack.chunks[y].is_none() { stack.chunks[y].replace(f()); }

        stack.chunks[y].as_mut().ok_or(ChunkAccessError::ChunkDoesNotExist)
    }

    fn remove_chunk(&mut self, position: IVec3) -> ChunkAccessResult<Option<ChunkType>> {
        let (x, y, z) = Self::get_check_position(position, self.height)?;
        let hash = Self::generate_hash(x, z);

        let stack_index = self.hash_to_index(hash);

        match stack_index {
            Some(stack_index) => {
                Ok(self.chunk_stacks[stack_index].chunks[y as usize].take())
            },
            None => { Err(ChunkAccessError::ChunkDoesNotExist) }
        }
    }

    fn get_nearby_chunks(&self, position: IVec3) -> Vec<Option<&ChunkType>> {
        DIRECTIONS.iter().map(|direction| {
                let position = direction.get_int_vector() + position;
                self.get_chunk(position).ok()
            }).collect()
    }
}

pub struct ChunkStoragePlanarLimited<ChunkType> {
    chunks: Vec<ChunkType>,
}

impl<ChunkType> ChunkStoragePlanarLimited<ChunkType> {
    pub fn new() -> Self {
        Self {
            chunks: vec![]
        }
    }
}

impl<ChunkType> ChunkStorageTrait<ChunkType> for ChunkStoragePlanarLimited<ChunkType> {
    fn set_chunk(&mut self, chunk: ChunkType, position: IVec3) -> ChunkAccessResult<()> {
        todo!()
    }

    fn get_chunk(&self, position: IVec3) -> ChunkAccessResult<&ChunkType> {
        todo!()
    }

    fn get_chunk_mut(&mut self, position: IVec3) -> ChunkAccessResult<&mut ChunkType> {
        todo!()
    }

    fn get_or_create_chunk<F: FnOnce() -> ChunkType>(&mut self, position: IVec3, f: F) -> ChunkAccessResult<&mut ChunkType> {
        todo!()
    }

    fn remove_chunk(&mut self, position: IVec3) -> ChunkAccessResult<Option<ChunkType>> {
        todo!()
    }

    fn get_nearby_chunks(&self, position: IVec3) -> Vec<Option<&ChunkType>> {
        todo!()
    }
}

pub struct ChunkStorageCubic<ChunkType> {
    chunks: Vec<ChunkType>,
}

impl<ChunkType> ChunkStorageCubic<ChunkType> {
    pub fn new() -> Self {
        Self {
            chunks: vec![]
        }
    }
}

impl<ChunkType> ChunkStorageTrait<ChunkType> for ChunkStorageCubic<ChunkType> {
    fn set_chunk(&mut self, chunk: ChunkType, position: IVec3) -> ChunkAccessResult<()> {
        todo!()
    }

    fn get_chunk(&self, position: IVec3) -> ChunkAccessResult<&ChunkType> {
        todo!()
    }

    fn get_chunk_mut(&mut self, position: IVec3) -> ChunkAccessResult<&mut ChunkType> {
        todo!()
    }

    fn get_or_create_chunk<F: FnOnce() -> ChunkType>(&mut self, position: IVec3, f: F) -> ChunkAccessResult<&mut ChunkType> {
        todo!()
    }

    fn remove_chunk(&mut self, position: IVec3) -> ChunkAccessResult<Option<ChunkType>> {
        todo!()
    }

    fn get_nearby_chunks(&self, position: IVec3) -> Vec<Option<&ChunkType>> {
        todo!()
    }
}
