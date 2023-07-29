use std::{sync::Mutex, ops::AddAssign};
use rustc_hash::FxHashMap as HashMap;

/// This module represents the data types for a chunk, which is defined as a column of 'chunk sections',
/// each of which stores the data for block, light, and metadata in an array of 16^3 elements, and
/// chunks are a stack of 16* sections (8 for legacy versions, and I believe 32 for most modern
/// versions)
///
/// The feature 'large_chunks' will enable an expeimental mode where chunks are 32^3 to house a 2x2
/// of chunks or a 2^3 of chunk sections for performance reasons
///
/// Chunks will treat their data as elements in the range of 0..ChunkSectionAxisSize for XZ and
/// 0..chunk_sections*ChunkSectionAxisSize for Y with no concept of negative positions
use ultraviolet::UVec3;

use crate::util::nibble;

/// These constants defines the overall size of a chunk, and to keep in line with minecraft, it will
/// be 16 or a power of 2

/**
 *  The total number of elements along an axis of a chunk
 *  16 by default, 32 on feature large_chunks
 */
pub const CHUNK_SECTION_AXIS_SIZE: usize = if cfg!(feature = "large_chunks") {
    32
} else {
    16
};

/**  
 * The size of an axis of the chunk minus one
 * 15 by default, 31 on feature large_chunks
 */
pub const CHUNK_SECTION_AXIS_SIZE_M1: usize = CHUNK_SECTION_AXIS_SIZE - 1;
/**
 *  The total number of elements in a plane in a chunk
 *  Represents 16^2 by default, and 32^2 on feature large_chunks
 */
const CHUNK_SECTION_PLANE_SIZE: usize = CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

/**
 *  The total number of elements in a chunk
 *
 */
const CHUNK_SECTION_DIMENSION_SIZE: usize =
    CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

pub type TBlockData = u16;

/// The chunk sections will be stored as vectors to be managed on the heap, but never be resized
pub type TLightData = u8;
type TChunkLightStorage = Vec<TLightData>;

type TGlobalId = usize;
type TLocalId = u16;
type TLocalToGlobalMap = Vec<TGlobalId>;
type TGlobalToLocalMap = HashMap<TGlobalId, TLocalId>;

#[derive(PartialEq, PartialOrd)]
enum BlockStorage {
    Empty {},
    Nibble { blocks: Vec<u8> },
    Byte { blocks: Vec<u8> },
    Short { blocks: Vec<u16> },
}

impl BlockStorage {
    fn get_local_id(&self, index: usize) -> TLocalId {
        match self {
            Self::Empty {} => { 0 },
            Self::Nibble { blocks } => { nibble::nibble_get(blocks, index).into() },
            Self::Byte { blocks } => { blocks[index].into() },
            Self::Short { blocks } => { blocks[index].into() }
        }
    }

    fn get_limit(&self) -> usize {
        match self {
            Self::Empty {} => { 0 },
            Self::Nibble { .. } => { 16 },
            Self::Byte { .. } => { 256 },
            Self::Short { .. } => { 4096 }
        }
    }

    fn is_empty(&self) -> bool {
        self.get_limit() == 0
    }

    fn is_nibble(&self) -> bool {
        self.get_limit() == 16
    }

    fn is_byte(&self) -> bool {
        self.get_limit() == 256
    }

    fn is_short(&self) -> bool {
        self.get_limit() == 4096
    }

    fn upgrade(&mut self) {
        if self.is_empty() {
            // log::warn!("Empty to Nibble Upgrade!");
            *self = Self::Nibble { blocks: vec![0; 2048] }
        }
        if self.is_nibble() {
            // log::warn!("Nibble to Byte Upgrade!");
            let mut new_data = Self::Byte { blocks: vec![0; 4096] };
            for index in 0..CHUNK_SECTION_DIMENSION_SIZE {
                new_data.set_local_id(index, self.get_local_id(index));
            }
            *self = new_data;
        }
        if self.is_byte() {
            // log::warn!("Byte to Short Upgrade!");
            let mut new_data = Self::Short { blocks: vec![0; 4096] };
            for index in 0..CHUNK_SECTION_DIMENSION_SIZE {
                new_data.set_local_id(index, self.get_local_id(index));
            }
            *self = new_data;
        }
    }

    fn downgrade(&mut self) {
        // log::warn!("Downgrade!");
        if self.is_nibble() {
            *self = Self::Empty {};
        }
        if self.is_byte() {
            let mut new_data = Self::Nibble { blocks: vec![0; 2048] };
            for index in 0..CHUNK_SECTION_DIMENSION_SIZE {
                new_data.set_local_id(index, self.get_local_id(index));
            }
            *self = new_data;
        }
        if self.is_short() {
            let mut new_data = Self::Byte { blocks: vec![0;4096] };
            for index in 0..CHUNK_SECTION_DIMENSION_SIZE {
                new_data.set_local_id(index, self.get_local_id(index));
            }
            *self = new_data;
        }
    }

    fn set_local_id(&mut self, index: usize, local_id: TLocalId) {
        match self {
            Self::Empty {} => {},
            Self::Nibble { blocks } => { nibble::nibble_set(blocks, index, local_id as u8) },
            Self::Byte { blocks } => { blocks[index] = local_id as u8 },
            Self::Short { blocks } => { blocks[index] = local_id }
        }
    }

    fn get_block(&self, index: usize, local_to_global_map: &TLocalToGlobalMap) -> usize {
        let local_id = self.get_local_id(index);
        // log::warn!("Getting block of {}L, map is {:?}", local_id, local_to_global_map);
        local_to_global_map[local_id as usize]
    }

    fn set_block(&mut self, index: usize, block: TGlobalId, local_to_global_map: &mut TLocalToGlobalMap, global_to_local_map: &mut TGlobalToLocalMap) {

        if self.is_empty() && block != 0 {
            // upgrade and return
            self.upgrade();
        } else if self.is_empty() { // adding air to air
            return;
        }

        let local_id = global_to_local_map.get(&block).cloned().unwrap_or_else(|| {
            // find next local id
            let new_local_id = local_to_global_map.len() as u16;
            global_to_local_map.insert(block, new_local_id);
            local_to_global_map.insert(new_local_id as usize, block);
            // log::warn!("Mapping {}L to {}G", new_local_id, block);
            new_local_id
        });
        
        // Expander/Shrinker
        let container_limit = self.get_limit();

        let block_count = local_to_global_map.len();
        if block_count > container_limit {
            // upgrade
            self.upgrade();
        } else if block_count <= container_limit / 16 {
            // downgrade
            self.downgrade();
        }

        // let old_local_id = self.get_local_id(index);
        self.set_local_id(index, local_id);

        /*  This is actually a performance tank lol
        // Optimizer
        let old_local_id_count = self.count_local_id(old_local_id);
        if old_local_id_count == 0 {
            // we've replaced all instances of the old id, swap ids
            self.set_local_id(index, old_local_id);
            local_to_global_map.swap_remove(old_local_id as usize);
        }
        */
    }

}

pub struct Chunk {
    /// Self explanitory
    block_storage: BlockStorage,
    /// Maps the value stored in the block storage to a state id
    local_to_global_map: TLocalToGlobalMap,
    /// Maps the state id to a value for the block storage
    global_to_local_map: TGlobalToLocalMap,
    /// Contains both skylight and blocklight
    /// Represents the inverse maximum skylight value of the chunk, so where a skylight value would
    /// only be able to be a maximum of 2, the stored value would be 13, and be calculated as
    /// (skylight(15) - lightmap_value(13)) = 2 | 14 - 13 = 1 | 13 - 13 = 0 | [0, 12] < 13 = 0.
    lightmap: TChunkLightStorage,
    /// Chunk data has been changed, related constructs need to be rebuilt
    dirty: Mutex<bool>,
}

impl Chunk {
    pub fn is_dirty(&self) -> bool { *self.dirty.lock().unwrap() }
    pub fn set_dirty(&self, dirty: bool) { *self.dirty.lock().unwrap() = dirty; }

    /// Create and return an empty chunk section for generation
    pub fn create_empty() -> Self {
        let lightmap = vec![0; CHUNK_SECTION_DIMENSION_SIZE];
        let block_storage = BlockStorage::Empty {  };
        let local_to_global_map = vec![0]; // default map air to 0
        let global_to_local_map = TGlobalToLocalMap::default();
        Self { block_storage, local_to_global_map, global_to_local_map, lightmap, dirty: Mutex::new(false) }
    }

    /// Get the index of block in storage from a 3d position
    fn calc_element_index_from_pos(x: u32, y: u32, z: u32) -> usize {
        (y as usize * CHUNK_SECTION_PLANE_SIZE)
            + (x as usize * CHUNK_SECTION_AXIS_SIZE)
            + z as usize
    }

    /// Get the 3d position of the block relative to the chunk
    fn calc_element_pos_from_index(index: usize) -> (u32, u32, u32) {
        (
            (index / CHUNK_SECTION_AXIS_SIZE) as u32,
            (index / CHUNK_SECTION_PLANE_SIZE) as u32,
            (index % CHUNK_SECTION_AXIS_SIZE) as u32,
        )
    }

    /// Get the data of a block from an index
    pub fn get_block_at_index(&self, index: usize) -> TBlockData {
        return self.block_storage.get_block(index, &self.local_to_global_map).try_into().unwrap();
    }

    /// Get the data of an element from an unsigned 3d position
    pub fn get_block_at_pos(&self, x: u32, y: u32, z: u32) -> TBlockData {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.get_block_at_index(index)
    }

    /// Get the data of an element from an unsigned 3d vector
    pub fn get_block_at_vec(&self, pos: UVec3) -> TBlockData {
        self.get_block_at_pos(pos.x, pos.y, pos.z)
    }

    /// Set the data of an element from an index
    fn set_block_at_index(&mut self, index: usize, data: TBlockData) {
        self.block_storage.set_block(index, data.into(), &mut self.local_to_global_map, &mut self.global_to_local_map)
    }

    /// Set the data of an element from an unsigned 3d position
    pub fn set_block_at_pos(&mut self, x: u32, y: u32, z: u32, data: TBlockData) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.set_block_at_index(index, data);
    }

    /// Set the data of an element from an unsigned 3d vector
    pub fn set_block_at_vec(&mut self, pos: UVec3, data: TBlockData) {
        self.set_block_at_pos(pos.x, pos.y, pos.z, data);
    }

    pub fn get_light_at_pos(&self, x: u32, y: u32, z: u32) -> (u8, u8) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        let light_data = self.lightmap[index];
        (light_data & 0b00001111, (light_data >> 4) & 0b00001111)
    }
    
    pub fn get_light_at_vec(&self, pos: UVec3) -> (TLightData, TLightData) {
        self.get_light_at_pos(pos.x, pos.y, pos.z)
    }

    pub fn set_skylight_at_pos(&mut self, x: u32, y: u32, z: u32, light_value: TLightData) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        let bs_light = self.lightmap[index];
        let b_light = bs_light & 0b11110000;
        let s_light = light_value & 0b00001111;
        self.lightmap[index] = b_light | s_light;
    }

    pub fn set_skylight_at_vec(&mut self, pos: UVec3, light_value: TLightData) {
        self.set_skylight_at_pos(pos.x, pos.y, pos.z, light_value);
    }

    pub fn set_blocklight_at_pos(&mut self, x: u32, y: u32, z: u32, light_value: TLightData) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        let bs_light = self.lightmap[index];
        let b_light = light_value << 4;
        let s_light = bs_light & 0b00001111;
        self.lightmap[index] = b_light | s_light;
    }

    pub fn set_blocklight_at_vec(&mut self, pos: UVec3, light_value: TLightData) {
        self.set_blocklight_at_pos(pos.x, pos.y, pos.z, light_value);
    }

    pub fn get_global_to_local_map(&self) -> &TGlobalToLocalMap {
        &self.global_to_local_map
    }

    pub fn get_local_to_global_map(&self) -> &TLocalToGlobalMap {
        &self.local_to_global_map
    }

}

/// The type of value stored in the heighmap - a tuple of topmost (opaque, transparent) - will help
/// with 
pub type ChunkHeightmapType = (i32, i32);
