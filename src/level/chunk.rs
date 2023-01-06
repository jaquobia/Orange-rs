/// This module represents the data types for a chunk, which is defined as a column of 'chunk sections',
/// each of which stores the data for blocks, light, and metadata in an array of 16^3 elements, and
/// chunks are a stack of 16* sections (8 for legacy versions, and I believe 32 for most modern
/// versions)
///
/// The feature 'large_chunks' will enable an expeimental mode where chunks are 32^3 to house a 2x2
/// of chunks or a 2^3 of chunk sections for performance reasons
///
/// Chunks will treat their data as elements in the range of 0..ChunkSectionAxisSize for XZ and
/// 0..chunk_sections*ChunkSectionAxisSize for Y with no concept of negative positions


use ultraviolet::{UVec3, IVec2, Vec3, IVec3};
use wgpu::RenderPass;

use crate::{rendering::{mesh::Mesh, tessellator::TerrainTessellator}, registry::Register, block::Block, direction::{Direction, DIRECTIONS}};

/// These constants defines the overall size of a chunk section, and to keep in line with minecraft, it will
/// be 16 or a power of 2

/**
 *  The total number of elements along an axis of a chunk section
 */

pub const CHUNK_SECTION_AXIS_SIZE: usize = if cfg!(feature = "large_chunks") { 32 } else { 16 };

pub const CHUNK_SECTION_AXIS_SIZE_M1: usize = CHUNK_SECTION_AXIS_SIZE - 1;
/**
 *  The total number of elements in a plane in a chunk section
 */
const CHUNK_SECTION_PLANE_SIZE: usize = CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

/**
 *  The total number of elements in a chunk section
 */
const CHUNK_SECTION_DIMENSION_SIZE: usize = CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE * CHUNK_SECTION_AXIS_SIZE;

/// Put some documentation here about the decision for this type and how its relevant
type ChunkDataType = u64;

/// The chunk sections will be stored as vectors to be managed on the heap, but never be resized
// type ChunkSectionDataStorageType = [ChunkDataType; ChunkSectionDimension];
type ChunkSectionDataStorageType = Vec<ChunkDataType>; 

pub struct ChunkSection {
    data: ChunkSectionDataStorageType,
    mesh: Option<Mesh>, // Mesh could be yet to be baked
}

impl ChunkSection {

    /// Create and return an empty chunk section
    fn create_empty() -> Self {
        let data: ChunkSectionDataStorageType = vec![1; CHUNK_SECTION_DIMENSION_SIZE];
        Self {
            data,
            mesh: None,
        }
    }

    /// Create and return a chunk section from existing data
    fn from_data(data: ChunkSectionDataStorageType) -> Self {
        Self {
            data,
            mesh: None,
        }
    }

    fn calc_element_index_from_pos(x: u32, y: u32, z: u32) -> usize {
        (y as usize * CHUNK_SECTION_PLANE_SIZE) + (x as usize * CHUNK_SECTION_AXIS_SIZE) + z as usize
    }

    fn calc_element_pos_from_index(index: usize) -> (u32, u32, u32) {
        ( (index / CHUNK_SECTION_AXIS_SIZE) as u32, (index / CHUNK_SECTION_PLANE_SIZE) as u32, (index % CHUNK_SECTION_AXIS_SIZE) as  u32 )
    }

    /// Get the data of an element from an index
    fn get_index(&self, index: usize) -> ChunkDataType {
        return self.data[index];
    }

    /// Get the data of an element from an unsigned 3d position
    fn get_pos(&self, x: u32, y: u32, z: u32) -> ChunkDataType {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.get_index(index)
    }

    /// Get the data of an element from an unsigned 3d vector
    fn get_vec(&self, pos: UVec3) -> ChunkDataType {
        self.get_pos(pos.x, pos.y, pos.z)
    }

    /// Set the data of an element from an index
    fn set_index(&mut self, index: usize, data: ChunkDataType) {
        self.data[index] = data;
    }

    /// Set the data of an element from an unsigned 3d position
    fn set_pos(&mut self, x: u32, y: u32, z: u32, data: ChunkDataType) {
        let index = Self::calc_element_index_from_pos(x, y, z);
        self.set_index(index, data);
    }

    /// Set the data of an element from an unsigned 3d vector
    fn set_vec(&mut self, pos: UVec3, data: ChunkDataType) {
        self.set_pos(pos.x, pos.y, pos.z, data);
    }

    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.mesh.replace(mesh);
    }

    pub fn get_mesh(&self) -> Option<&Mesh> {
        self.mesh.as_ref()
    }

    pub fn get_mesh_mut(&mut self) -> Option<&mut Mesh> {
        self.mesh.as_mut()
    }

}

pub struct Chunk {
    /// The Sections of a chunk
    sections: Vec<ChunkSection>,
    /// The signed 3d vector of the chunks position
    position: IVec2,
}

impl Chunk {

    pub fn new(position: IVec2, num_section: usize) -> Self {
        let mut sections = Vec::with_capacity(num_section);
        for i in 0..(num_section-1) {
            sections.push(ChunkSection::create_empty() );
        }
        Self {
            sections,
            position,
        }
    }

    /// Get the data of the chunk at an unsigned 3d position
    /// Returns the data if the chunksection is present, or 0 (air) if not
    pub fn get_block_at_pos(&self, x: u32, y: u32, z: u32) -> ChunkDataType {
        let section_index = y as usize / CHUNK_SECTION_AXIS_SIZE;
        let section = self.sections.get(section_index);
        // fancy ternery + unwrap operation
        return if let Some(section) = section { section.get_pos(x, y % (CHUNK_SECTION_AXIS_SIZE as u32), z) } else { 0 };
    }

    /// Get the data of the chunk as an unsigned 3d vector
    pub fn get_block_at_vec(&self, pos: UVec3) -> ChunkDataType {
        self.get_block_at_pos(pos.x, pos.y, pos.z)
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

    pub fn get_pos(&self) -> IVec2 { self.position }

    pub fn chunk_data_helper(data: u64) -> (usize, u64) {
        const block_magic_number: u64 = 0b111111111111;
        let chunk_block = (data & block_magic_number) as usize;
        let metadata = (data & !block_magic_number) >> 12;
        (chunk_block, metadata)
    }

    pub fn tesselate(&mut self, tesselator: &mut TerrainTessellator, queue: &wgpu::Queue, device: &wgpu::Device, blocks: &Register<Block>) {
        let (chunk_x, chunk_z) = (self.position.x << 4, self.position.y << 4);
        let air_id = blocks.get_index_from_identifier("air");
        let mut section_index: f32 = -1.0;
        for section in &mut self.sections {
            section_index += 1.0;
            let section_position = Vec3::new(chunk_x as f32, section_index * CHUNK_SECTION_AXIS_SIZE as f32, chunk_z as f32);
            for y in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                for x in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                    for z in 0..CHUNK_SECTION_AXIS_SIZE as u32 {
                        let pos_vec = UVec3::new(x, y, z);
                        let pos_ivec = IVec3::new(x as i32, y as i32, z as i32);

                        let position = Vec3::new(x as f32, y as f32, z as f32);
                        let position_extent = position + Vec3::one();
                        let chunk_data = section.get_vec(pos_vec);

                        let (block_id, metadata) = Self::chunk_data_helper(chunk_data);
                        if block_id == air_id { continue; }

                        let block = blocks.get_element_from_index(block_id);
                        let mut occlusions: [bool; 6] = [false; 6];
                        let textures: [u32; 6] = if let Some(block) = block.as_ref() {
                            [block.texture_index() as u32; 6]
                        } else { [0;  6] };

                        for dir in &DIRECTIONS {
                            let dir_index = dir.ordinal();
                            let new_pos = pos_ivec + dir.get_int_vector();
                            if new_pos.x < 0 || new_pos.x > CHUNK_SECTION_AXIS_SIZE_M1 as i32 ||
                                new_pos.y < 0 || new_pos.y > CHUNK_SECTION_AXIS_SIZE_M1 as i32 ||
                                new_pos.z < 0 || new_pos.z > CHUNK_SECTION_AXIS_SIZE_M1 as i32 {
                                occlusions[dir_index] = true; // Get information from neighbor
                                                                  // chunk
                                continue;
                            }
                            let chunk_data = section.get_vec(UVec3::new(new_pos.x as u32, new_pos.y as u32, new_pos.z as u32));
                            let (block_id, metadata) = Self::chunk_data_helper(chunk_data);
                            if let Some(block) = blocks.get_element_from_index(block_id).as_ref() {
                                occlusions[dir_index] = block.is_transparent();
                            }
                            
                        }

                        tesselator.cuboid(position + section_position, Vec3::one(), textures, &occlusions);
                    }
                }
            }

            if section.mesh.is_none() {
                let mesh = tesselator.build(device);
                section.mesh.replace(mesh);
            } else {
                let mesh = section.mesh.as_mut().unwrap();
                tesselator.into_mesh(queue, mesh);
            }
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for section in & self.sections {
            if let Some(mesh) = section.get_mesh() {
                mesh.draw(render_pass);
            }
        } 
    }
}
