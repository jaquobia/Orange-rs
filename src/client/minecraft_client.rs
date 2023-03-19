use std::{cell::RefCell};

use super::{rendering::{mesh::Mesh, world_renders::WorldRenderer}, gui::screen::Screen};
use crate::{
    util::pos::{ChunkPos},
};
use ultraviolet::{IVec3};
use wgpu::RenderPass;

pub type ClientChunkStorage = Option<Mesh>;

pub struct MinecraftClient {
    player_level_id: usize,
    pub world_render: WorldRenderer,

    active_screen : RefCell<Option<Box<dyn Screen>>>,
}

impl MinecraftClient {

    pub fn new(num_sections: u32) -> Self {
        Self {
            player_level_id: 0,
            world_render: WorldRenderer::new(num_sections),

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

    pub fn draw_chunks<'a>(&'a self, min_extent: ChunkPos, max_extent: ChunkPos, render_pass: &mut RenderPass<'a>) {
        for x in min_extent.x..=max_extent.x {
            for z in min_extent.y..=max_extent.y {
                let vec16 = IVec3::new(16, 16, 16);
                if let Some(chunk) = self.world_render.get_cache().get_chunk_pos(x, z) {
                    let mut mesh_index = 0i32;
                    for mesh in chunk.get_sections() {
                        let chunk_pos_min = IVec3::new(x << 4, mesh_index << 4, z << 4);
                        let chunk_pos_max = chunk_pos_min + vec16;

                        match mesh {
                            Some(mesh) => mesh.draw(render_pass),
                            None => { },
                        };
                        mesh_index += 1;
                    }
                }
            }
        }
    }

    pub fn process_chunks(&mut self, min_extent: ChunkPos, max_extent: ChunkPos) {
        for chunk in self.world_render.get_chunks() {
            if !chunk.in_range(min_extent, max_extent) {
                chunk.mark_for_removal();
            }
        }
        self.world_render.remove_marked_chunks();
    }

}
