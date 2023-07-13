use std::cell::Cell;

use crate::util::pos::ChunkPos;

use super::rendering::mesh::Mesh;

pub struct ClientChunk {
    sections: Vec<Option<Mesh>>,
    position: ChunkPos,
    /// Used to indicate when a chunk should be removed next time client chunks are processed
    marked_for_removal: Cell<bool>,
    /// Used to indicate whether a chunk has been queued to be meshed, and to not constantly ask for one   
    /// Only set to true when the client cant find the chunk and makes one.
    marked_for_meshing: Cell<bool>,
    /// Used to indicate whether a chunk has been meshed or not
    /// Only set to false on initial chunk generation
    meshed: Cell<bool>,
}

impl ClientChunk {
    pub fn new(position: ChunkPos, chunk_height: usize) -> Self {
        let mut sections = Vec::with_capacity(chunk_height);
        for _ in 0..chunk_height {
            sections.push(None);
        }
        Self {
            sections,
            position,
            marked_for_removal: Cell::new(false),
            marked_for_meshing: Cell::new(false),
            meshed: Cell::new(false),
        }
    } 

    pub fn in_range(&self, min_extent: ChunkPos, max_extent: ChunkPos) -> bool {
        return self.position.x <= max_extent.x
            && self.position.y <= max_extent.y
            && self.position.x >= min_extent.x
            && self.position.y >= min_extent.y;
    }

    pub fn mark_for_removal(&self) {
        self.marked_for_removal.set(true);
    }

    pub fn unmark_for_removal(&self) {
        self.marked_for_removal.set(false);
    }

    pub fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal.get()
    }

    pub fn mark_for_meshing(&self) {
        self.marked_for_meshing.set(true);
    }

    pub fn unmark_for_meshing(&self) {
        self.marked_for_meshing.set(false);
    }

    pub fn is_marked_for_meshing(&self) -> bool {
        self.marked_for_meshing.get()
    }

    pub fn mark_meshed(&self) {
        self.meshed.set(true);
    }

    pub fn unmark_meshed(&self) {
        self.meshed.set(false);
    }

    pub fn is_meshed(&self) -> bool {
        self.meshed.get()
    }

    pub fn get_sections(&self) -> &Vec<Option<Mesh>> {
        &self.sections
    }

    pub fn get_section(&self, section: usize) -> &Option<Mesh> {
        &self.sections[section]
    }

    pub fn set_section(&mut self, section: usize, mesh: Option<Mesh>) {
        self.sections[section] = mesh;
    }
}
