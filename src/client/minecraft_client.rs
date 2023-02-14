use std::{collections::VecDeque, cell::RefCell};

use super::{rendering::{mesh::Mesh, tessellator::TerrainTessellator, world_renders::WorldRenderer}, gui::screen::Screen};
use crate::{
    block::Block,
    world::{
        chunk::CHUNK_SECTION_AXIS_SIZE,
        dimension::{Dimension, DimensionChunkDescriptor}, 
    },
    registry::Register,
    util::pos::ChunkPos,
};
use ultraviolet::Vec3;
use wgpu::RenderPass;

pub type ClientChunkStorage = Option<Mesh>;

pub struct MinecraftClient {
    player_level_id: usize,
    world_render: WorldRenderer,

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

    pub fn tesselate_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        tessellator: &mut TerrainTessellator,
        device: &wgpu::Device,
        blocks: &Register<Block>,
        level: &Dimension,
        ) -> Result<(), ()> {
        
        let (chunk_x, chunk_z) = (chunk_pos * ChunkPos::new(CHUNK_SECTION_AXIS_SIZE as i32, CHUNK_SECTION_AXIS_SIZE as i32)).into();

        let chunk = {
            // let level = self.get_player_dimension();
            // Level can't be obtained? Put back on queue, and try again later
            // if level.is_none() {
            //     return Err(());
            // }
            // let level = level.unwrap();

            let chunk = level.get_chunk_at_vec(chunk_pos);

            // If chunk is none, nothing to build, skip
            if chunk.is_none() {
                return Ok(());
            }
            chunk.unwrap()
        };

        let mut section_index: usize = 0;

        let mut meshes: Vec<Mesh> = vec![];
        for section in chunk.get_sections() {
            let section_position = Vec3::new(
                chunk_x as f32,
                (section_index * CHUNK_SECTION_AXIS_SIZE) as f32,
                chunk_z as f32,
                );
            tessellator.tesselate_chunk_section(section, section_position, blocks);
            let mesh = tessellator.build(device);
            meshes.push(mesh);
            section_index += 1;
        }
        self.world_render.construct_chunk(meshes, chunk_pos);
        return Ok(());
    }

    pub fn process_chunks(&mut self, min_extent: ChunkPos, max_extent: ChunkPos, tessellation_queue: &mut VecDeque::<DimensionChunkDescriptor>) {
        for chunk in self.world_render.get_chunks() {
            let chunk_in_range = chunk.in_range(min_extent, max_extent);
            if !chunk_in_range {
                // println!("Chunk not in range!");
                chunk.mark_for_removal();
            }
        }
        self.world_render.remove_marked_chunks();
        for x in min_extent.x..=max_extent.x {
            for z in min_extent.y..=max_extent.y {
                let pos = ChunkPos::new(x, z);
                if self.world_render.get_cache().get_chunk_vec(pos).is_none() {
                    self.world_render.construct_chunk_empty(pos);
                    tessellation_queue.push_back((0, pos));
                }
            }
        }
    }

}
