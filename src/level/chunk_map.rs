use std::collections::HashMap;

use ultraviolet::IVec2;

pub struct ChunkMap<T> {
    chunks: Vec<T>,
    map_pos_to_index: HashMap<i64, usize>,
}

const CHUNK_ALLOC_MAGIC_NUMBER: usize = 21 * 21;

impl<T> ChunkMap<T> {
    pub fn new() -> Self {
        let chunks = Vec::with_capacity(CHUNK_ALLOC_MAGIC_NUMBER);

        Self {
            chunks,
            map_pos_to_index: HashMap::new(),
        }
    }

    fn inner_chunk_pos_to_hash(x: i32, z: i32) -> i64 {
        let x = x as i64;
        let z = z as i64;
        return x + (z << 32);
    }

    fn inner_hash_to_index(&self, hash: i64) -> Option<usize> {
        self.map_pos_to_index
            .get(&hash)
            .and_then(|&index| Some(index)) // Make not a reference to
                                            // not continuously borrow
                                            // self.map_pos_to_index
    }

    fn inner_pos_to_index(&self, x: i32, z: i32) -> Option<usize> {
        self.inner_hash_to_index(Self::inner_chunk_pos_to_hash(x, z))
    }

    pub fn get_chunk_pos(&self, x: i32, z: i32) -> Option<&T> {
        match self.inner_pos_to_index(x, z) {
            Some(index) => self.chunks.get(index),
            None => None,
        }
    }

    pub fn get_chunk_pos_mut(&mut self, x: i32, z: i32) -> Option<&mut T> {
        match self.inner_pos_to_index(x, z) {
            Some(index) => self.chunks.get_mut(index),
            None => None,
        }
    }

    pub fn get_chunk_vec(&self, pos: IVec2) -> Option<&T> {
        self.get_chunk_pos(pos.x, pos.y)
    }

    pub fn get_chunk_vec_mut(&mut self, pos: IVec2) -> Option<&mut T> {
        self.get_chunk_pos_mut(pos.x, pos.y)
    }

    pub fn set_chunk(&mut self, pos: IVec2, chunk: Option<T>) {
        let hash = Self::inner_chunk_pos_to_hash(pos.x, pos.y);
        let index = self.inner_hash_to_index(hash);

        // We have no index, there is no chunk as this pos in storage
        if index.is_none() {
            // Add chunk to storage
            if chunk.is_some() {
                self.chunks.push(chunk.unwrap());
                self.map_pos_to_index.insert(hash, self.chunks.len() - 1);
            }
        } else {
            // There is a chunk already here
            let index = index.unwrap();
            if chunk.is_some() {
                // Replace the chunk
                self.chunks[index] = chunk.unwrap();
            } else {
                // delete the chunk
                self.map_pos_to_index.remove(&hash);
                self.chunks.remove(index);
            }
        }
    }

    pub fn delete_chunk(&mut self, pos: IVec2) {
        self.set_chunk(pos, None);
    }
}
