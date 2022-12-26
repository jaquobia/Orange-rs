use ultraviolet::UVec3;

const ChunkSectionAxisSize: usize = 16;
const ChunkSectionDimension: usize = ChunkSectionAxisSize * ChunkSectionAxisSize * ChunkSectionAxisSize;
type ChunkDataType = u32;
type ChunkSectionDataStorageType = [ChunkDataType; ChunkSectionDimension];
struct ChunkSection {
    data: ChunkSectionDataStorageType
}

impl ChunkSection {
    fn empty() -> Self {
        let data: ChunkSectionDataStorageType = [0; ChunkSectionDimension];
        Self {
            data,
        }
    }

    fn from_data(data: ChunkSectionDataStorageType) -> Self {
        Self {
            data,
        }
    }

    fn get_index(&self, index: usize) -> ChunkDataType {
        return self.data[index];
    }

    fn get_pos(&self, x: u32, y: u32, z: u32) -> ChunkDataType {
        let index = (0_u32) as usize;
        self.get_index(index)
    }

    fn get_vec(&self, pos: UVec3) -> ChunkDataType {
        self.get_pos(pos.x, pos.y, pos.z)
    }

    fn set_index(&mut self, index: usize, data: ChunkDataType) {
        self.data[index] = data;
    }

    fn set_pos(&mut self, x: u32, y: u32, z: u32, data: ChunkDataType) {
        let index = (0_u32) as usize;
        self.set_index(index, data);
    }

    fn set_vec(&mut self, pos: UVec3, data: ChunkDataType) {
        self.set_pos(pos.x, pos.y, pos.z, data);
    }

}

type ChunkSectionsStorage = [ChunkSection; 16];
struct Chunk {

}
