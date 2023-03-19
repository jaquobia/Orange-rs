use std::{cell::RefCell};

use super::{rendering::{mesh::Mesh}, gui::screen::Screen};
use crate::{
    util::pos::{ChunkPos},
};
use crate::world::{ChunkStorage, ChunkStoragePlanar};

pub type ClientChunkStorage = Option<Mesh>;

pub struct MinecraftClient {
    player_level_id: usize,
    pub client_chunk_storage: ChunkStorage<Mesh>,

    active_screen : RefCell<Option<Box<dyn Screen>>>,
}

impl MinecraftClient {
    pub fn new(height: usize) -> Self {
        Self {
            player_level_id: 0,
            // client_chunk_storage: WorldRenderer::new(num_sections),
            client_chunk_storage: ChunkStorage::Planar(ChunkStoragePlanar::new(height)),
            active_screen: RefCell::new(None),
        }
    }

    pub fn on_screen<F: Fn(&dyn Screen)>(&self, f: F) -> bool {
        match self.active_screen.borrow().as_deref() {
            Some(screen) => {
                f(screen);
                return true;
            },
            None => {  },
        }

        return false;
    }

    pub fn on_screen_mut<F: FnMut(&dyn Screen)>(&self, mut f: F) -> bool {
        match self.active_screen.borrow_mut().as_deref() {
            Some(screen) => {
                f(screen);
                return true;
            },
            None => {  },
        }

        return false;
    }

    pub fn has_screen(&self) -> bool {
        return self.active_screen.borrow().as_deref().is_some();
    }

    // pub fn set_screen(&self, screen: Option<Box<dyn Screen>>) {
    //     self.active_screen.replace(screen);
    // }
    pub fn set_screen<S: Screen + 'static>(&self) {
        self.active_screen.replace(Some(Box::new(S::new())));
    }

    pub fn get_player_level_id(&self) -> usize {
        self.player_level_id
    }

    pub fn process_chunks(&mut self, min_extent: ChunkPos, max_extent: ChunkPos) {
        // for chunk in self.client_chunk_storage.get_chunks() {
        //     if !chunk.in_range(min_extent, max_extent) {
        //         chunk.mark_for_removal();
        //     }
        // }
        // self.client_chunk_storage.remove_marked_chunks();
    }

}
