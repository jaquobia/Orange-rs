use std::{collections::VecDeque, cell::RefCell};

use super::{rendering::{mesh::Mesh, tessellator::TerrainTessellator, world_renders::WorldRenderer}, gui::screen::Screen};
use crate::{
    block::Block,
    world::{
        chunk::{CHUNK_SECTION_AXIS_SIZE, Chunk},
    },
    registry::Register,
    util::pos::{ChunkPos, Position},
};
use ultraviolet::Vec3;
use wgpu::RenderPass;
use crate::util::pos::NewChunkPosition;
use crate::world::chunk::ChunkSection;

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
    pub fn set_screen<'a, S: Screen + 'static>(&'a self) {
        self.active_screen.replace(Some(Box::new(S::new())));
    }

    pub fn get_player_level_id(&self) -> usize {
        self.player_level_id
    }

    pub fn draw_chunk<'a>(&'a self, x: i32, z: i32, render_pass: &mut RenderPass<'a>) {
        if let Some(chunk) = self.world_render.get_cache().get_chunk_pos(x, z) {
            for mesh in chunk.get_sections() {
                match mesh {
                    Some(mesh) => mesh.draw(render_pass),
                    None => { },
                };
            }
        }
    }

    pub fn draw_chunks<'a>(&'a self, min_extent: ChunkPos, max_extent: ChunkPos, render_pass: &mut RenderPass<'a>) {
        for x in min_extent.x..=max_extent.x {
            for z in min_extent.y..=max_extent.y {
                self.draw_chunk(x, z, render_pass);
            }
        }
    }

    pub fn direct_tessellate_chunk(&mut self,
                                   tessellator: &mut TerrainTessellator,
                                   device: &wgpu::Device,
                                   blocks: &Register<Block>,
                                   chunk_pos: NewChunkPosition,
                                   chunk: &ChunkSection,
                                    // chunk_pos: ChunkPos,
) -> Result<(), ()> {
        let chunk_block_pos = chunk_pos.to_block_pos();
        let section_position = chunk_block_pos.to_entity_pos();
        let section_index = chunk_pos.vec.y as usize;
        tessellator.tessellate_chunk_section(chunk, section_position, blocks);
        let mesh = tessellator.build(device);
        self.world_render.set_section_mesh(mesh, chunk_pos.to_chunk_pos(), section_index);

        Ok(())
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
