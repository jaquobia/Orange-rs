use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

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
use ultraviolet::{IVec2, UVec3};

use crate::util::pos::{ChunkPos, InnerChunkPos, InnerChunkPosTrait};

/// These constants defines the overall size of a chunk section, and to keep in line with minecraft, it will
/// be 16 or a power of 2

/**
 *  The total number of elements along an axis of a chunk section
 */

pub const CHUNK_SECTION_AXIS_SIZE: usize = if cfg!(feature = "large_chunks") {
    32
} else {
    16
};

pub const CHUNK_SECTION_AXIS_SIZE_M1: usize = CHUNK_SECTION_AXIS_SIZE - 1;
/**
 *  The total number of elements in a plane in a chunk section
 */
const CHUNK_SECTION_PLANE_SIZE: usize = CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

/**
 *  The total number of elements in a chunk section
 */
const CHUNK_SECTION_DIMENSION_SIZE: usize =
    CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

/// 8 bits block id, 4 bits metadata, 4 bits blocklight
/// But in actuality, a u16 will be used because it is the smallest integer type that will encompass
/// The 16^3 and 32^3 region, representing possible blockstates
pub type ChunkDataType = u16;
pub type ChunkDataUnpackedType = (usize, ChunkDataType, ChunkDataType);

/// The chunk sections will be stored as vectors to be managed on the heap, but never be resized
type ChunkSectionDataStorageType = Vec<ChunkDataType>;
type LightmapType = u8;
type ChunkSectionLightmapStorageType = Vec<LightmapType>;

pub struct ChunkSection {
    /// The blockdata of the chunk, storing block light, metadata, blockid, and whatever else can
    /// fit into a u64. 
    /// The data outside of the id might be moved into a mojang inspired bitarray
    /// where the number of bits needed to store all the associated data is measured in bits, allocated by bytes, 
    /// and indexed by bit to byte conversions.
    data: ChunkSectionDataStorageType,
    /// Represents the inverse maximum skylight value of the chunk, so where a skylight value would
    /// only be able to be a maximum of 2, the stored value would be 13, and be calculated as
    /// (skylight(15) - lightmap_value(13)) = 2 | 14 - 13 = 1 | 13 - 13 = 0 | [0, 12] < 13 = 0.
    lightmap: ChunkSectionLightmapStorageType,
    dirty: Mutex<bool>,
}

impl ChunkSection {
    pub fn is_dirty(&self) -> bool { *self.dirty.lock().unwrap() }
    pub fn set_dirty(&self, dirty: bool) { *self.dirty.lock().unwrap() = dirty; }

    /// Create and return an empty chunk section for generation
    pub fn create_empty() -> Self {
        let data: ChunkSectionDataStorageType = vec![0; CHUNK_SECTION_DIMENSION_SIZE];
        let lightmap = vec![0; CHUNK_SECTION_DIMENSION_SIZE];
        Self { data, lightmap, dirty: Mutex::new(false) }
    }

    /// Create and return a chunk section from existing data
    fn from_data(data: ChunkSectionDataStorageType, lightmap: ChunkSectionLightmapStorageType) -> Self {
        Self { data, lightmap, dirty: Mutex::new(true) }
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
    fn get_index(&self, index: usize) -> ChunkDataType {
        return self.data[index];
    }

    /// Get the data of an element from an unsigned 3d position
    pub fn get_pos(&self, x: u32, y: u32, z: u32) -> ChunkDataType {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.get_index(index)
    }

    /// Get the data of an element from an unsigned 3d vector
    pub fn get_vec(&self, pos: UVec3) -> ChunkDataType {
        self.get_pos(pos.x, pos.y, pos.z)
    }

    /// Set the data of an element from an index
    fn set_index(&mut self, index: usize, data: ChunkDataType) {
        self.data[index] = data;
    }

    /// Set the data of an element from an unsigned 3d position
    pub fn set_pos(&mut self, x: u32, y: u32, z: u32, data: ChunkDataType) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.set_index(index, data);
    }

    /// Set the data of an element from an unsigned 3d vector
    pub fn set_vec(&mut self, pos: UVec3, data: ChunkDataType) {
        self.set_pos(pos.x, pos.y, pos.z, data);
    }

    pub fn get_lightmap_pos(&self, x: u32, y: u32, z: u32) -> LightmapType {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.lightmap[index]
    }

    pub fn set_lightmap_pos(&mut self, x: u32, y: u32, z: u32, lightmap_value: LightmapType) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.lightmap[index] = lightmap_value;
    }
    
    pub fn get_lightmap_vec(&self, pos: UVec3) -> LightmapType {
        let index = Self::calc_element_index_from_pos(pos.x, pos.y, pos.z);
        self.lightmap[index]
    }

    pub fn set_lightmap_vec(&mut self, pos: UVec3, lightmap_value: LightmapType) {
        let index = Self::calc_element_index_from_pos(pos.x, pos.y, pos.z);
        self.lightmap[index] = lightmap_value;
    }
}

/// The type of value stored in the heighmap - a tuple of topmost (opaque, transparent) - will help
/// with 
pub type ChunkHeightmapType = (i32, i32);
type ChunkHeightmapStorageType = Vec<ChunkHeightmapType>;

pub struct Chunk {
    /// The Sections of a chunk, stored as a stack of CHUNK_SECTION_AXIS_SIZE^3 regions of block
    sections: Vec<ChunkSection>,
    /// The signed 3d vector of the chunks position
    position: ChunkPos,
    /// The heightmap of the chunk, tells where the topmost transparent and opaque block of the world are located.
    heightmap: ChunkHeightmapStorageType,

    dirty: bool,
}

impl Chunk {
    pub fn new(position: ChunkPos, num_section: usize) -> Self {
        let mut sections = Vec::with_capacity(num_section);
        for _ in 0..num_section {
            sections.push(ChunkSection::create_empty());
        }
        let heightmap = vec![(0, 0); CHUNK_SECTION_PLANE_SIZE];
        Self { sections, position, heightmap, dirty: false }
    }

    /// Get the data of the chunk at an unsigned 3d position
    /// Returns the data if the chunksection is present, or 0 (air) if not
    pub fn get_block_at_pos(&self, x: u32, y: u32, z: u32, section_index: u32) -> ChunkDataType {
        let section = self.sections.get(section_index as usize);
        // fancy ternery + unwrap operation
        return if let Some(section) = section {
            section.get_pos(x, y, z)
        } else {
            0
        };
    }

    /// Get the data of the chunk as an unsigned 3d vector
    pub fn get_block_at_vec(&self, pos: InnerChunkPos) -> ChunkDataType {
        self.get_block_at_pos(pos.x(), pos.y(), pos.z(), pos.1)
    }

    /// Set the data of the chunk at an unsigned 3d position
    pub fn set_block_at_pos(&mut self, data: ChunkDataType, x: u32, y: u32, z: u32) {
        let section_index = y as usize / CHUNK_SECTION_AXIS_SIZE;
        let section = self.sections.get_mut(section_index);

        if let Some(section) = section {
            section.set_pos(x, y % (CHUNK_SECTION_AXIS_SIZE as u32), z, data)
        }
    }

    /// Set the data of the chunk at an unsigned 3d vector
    pub fn set_block_at_vec(&mut self, data: ChunkDataType, pos: UVec3) {
        self.set_block_at_pos(data, pos.x, pos.y, pos.z)
    }

    /// Get the lightmap value of the chunk as an unsigned 3d position
    pub fn get_lightmap_at_pos(&self, x: u32, y: u32, z: u32) -> LightmapType {
        let section_index = y as usize / CHUNK_SECTION_AXIS_SIZE;
        let section = self.sections.get(section_index);
        // fancy ternery + unwrap operation
        return if let Some(section) = section {
            section.get_lightmap_pos(x, y % (CHUNK_SECTION_AXIS_SIZE as u32), z)
        } else {
            0
        };
    }

    /// Get the lightmap value of the chunk as an unsigned 3d vector
    pub fn get_lightmap_at_vec(&self, pos: UVec3) -> LightmapType {
        self.get_lightmap_at_pos(pos.x, pos.y, pos.z)
    }

    /// Set the lightmap value of the chunk at an unsigned 3d position
    pub fn set_lightmap_at_pos(&mut self, data: LightmapType, x: u32, y: u32, z: u32) {
        let section_index = y as usize / CHUNK_SECTION_AXIS_SIZE;
        let section = self.sections.get_mut(section_index);

        if let Some(section) = section {
            section.set_lightmap_pos(x, y % (CHUNK_SECTION_AXIS_SIZE as u32), z, data)
        }
    }

    /// Set the lightmap value of the chunk at an unsigned 3d vector
    pub fn set_lightmap_at_vec(&mut self, data: LightmapType, pos: UVec3) {
        self.set_lightmap_at_pos(data, pos.x, pos.y, pos.z)
    }

    pub fn get_heightmap(&self, x: u32, z: u32) -> ChunkHeightmapType {
        let index = z as usize + (x as usize * CHUNK_SECTION_AXIS_SIZE);
        self.heightmap[index]
    }

    pub fn set_heightmap(&mut self, x: u32, z: u32, data: ChunkHeightmapType) {
        let index = z as usize + (x as usize * CHUNK_SECTION_AXIS_SIZE);
        self.heightmap[index] = data;
    }

    pub fn get_pos(&self) -> IVec2 {
        self.position
    }

    const BLOCK_MASK: ChunkDataType         = 0b0000000011111111;
    const META_MASK: ChunkDataType          = 0b0000111100000000;
    const BLOCK_LIGHT_MASK: ChunkDataType   = 0b1111000000000000;
    // const BLOCK_OFFSET: ChunkDataType = 0;
    const META_OFFSET: ChunkDataType = 8;
    const BLOCK_LIGHT_OFFSET: ChunkDataType = 12;


    pub fn chunk_data_helper(data: ChunkDataType) -> ChunkDataUnpackedType {
        let chunk_block = data & Chunk::BLOCK_MASK;
        let metadata = (data & Chunk::META_MASK) >> Chunk::META_OFFSET;
        let block_light = (data & Chunk::BLOCK_LIGHT_MASK) >> Chunk::BLOCK_LIGHT_OFFSET;
        (chunk_block as usize, metadata, block_light)
    }

    pub fn data_set_block(block_data: ChunkDataType, block: usize) -> ChunkDataType {
        (block_data & !Chunk::BLOCK_MASK) | (block as ChunkDataType & Chunk::BLOCK_MASK)
    }

    pub fn data_set_meta(block_data: ChunkDataType, meta: ChunkDataType) -> ChunkDataType {
        (block_data & !Chunk::META_MASK) | ((meta << Chunk::META_OFFSET) & Chunk::META_MASK)
    }

    pub fn data_set_block_light(block_data: ChunkDataType, block_light: ChunkDataType) -> ChunkDataType {
        (block_data & !Chunk::BLOCK_LIGHT_MASK) | ((block_light << Chunk::BLOCK_LIGHT_OFFSET) & Chunk::BLOCK_LIGHT_MASK)
    }

    pub fn get_sections(&self) -> &Vec<ChunkSection> {
        &self.sections
    }

    pub fn get_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}
