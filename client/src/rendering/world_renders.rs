use std::{collections::VecDeque, cell::RefCell, borrow::BorrowMut};

use orange_rs::level::{chunk_map::ChunkMap, dimension::DimensionChunkDescriptor};
use ultraviolet::IVec2;

use crate::client_chunk::ClientChunk;

use super::mesh::Mesh;

type WorldRenderChunkType = ClientChunk;

pub struct WorldRenderer {
    mesh_cache: ChunkMap<WorldRenderChunkType>,
    removal_queue: RefCell<VecDeque<IVec2>>,
    chunk_height: u32,
}

impl WorldRenderer {

    pub fn new(chunk_height: u32) -> Self {
        let mesh_cache = ChunkMap::new();
        Self {
            mesh_cache,
            removal_queue: RefCell::new(VecDeque::new()),
            chunk_height,
        }
    }

    pub fn construct_chunk(&mut self, mut meshes: Vec<Mesh>, pos: IVec2, chunk_height: usize) {
        let mut chunk = ClientChunk::new(pos, chunk_height);
        for i in 0..meshes.len() {
            chunk.set_section(i, meshes.pop());
        }
        chunk.mark_meshed();
        chunk.unmark_for_meshing();
        self.mesh_cache.set_chunk(pos, Some(chunk));
    }

    pub fn get_chunks(&self) -> &Vec<WorldRenderChunkType> {
        self.mesh_cache.chunks()
    }

    pub fn get_chunks_mut(&mut self) -> &mut Vec<WorldRenderChunkType> {
        self.mesh_cache.chunks_mut()
    }

    pub fn set_section_mesh(&mut self, mesh: Mesh, pos: IVec2, section: usize) {
        if let Some(chunk) = self.mesh_cache.get_chunk_pos_mut(pos.x, pos.y) {
           chunk.set_section(section, Some(mesh));
        }
    }

    pub fn remove_section_mesh(&mut self, pos: IVec2, section: usize) {
        if let Some(chunk) = self.mesh_cache.get_chunk_pos_mut(pos.x, pos.y) {
            chunk.set_section(section, None);
            
        }
    }

    pub fn draw_section_mesh<'a>(&'a self, pos: IVec2, section: usize, render_pass: &'a mut wgpu::RenderPass<'a>) {
        if let Some(chunk) = self.mesh_cache.get_chunk_pos(pos.x, pos.y) {
            if let Some(mesh) = &chunk.get_section(section) {
                mesh.draw(render_pass);
            } 
        }
    }

    pub fn draw_chunk_mesh<'a>(&'a self, pos: IVec2, render_pass: &mut wgpu::RenderPass<'a>, tessellation_queue: &mut VecDeque<DimensionChunkDescriptor>) {
        if let Some(chunk) = self.mesh_cache.get_chunk_pos(pos.x, pos.y) {
            for mesh in chunk.get_sections() {
                match mesh {
                    Some(mesh) => mesh.draw(render_pass),
                    None => { println!("no mesh, queueing"); tessellation_queue.push_back((0, pos)); },
                };
            }
        } else {
            // println!("no chunk, queueing");
            tessellation_queue.push_back((0, pos));
        }
    }

    pub fn mark_chunk_for_removal(&self, pos: IVec2) {
        self.removal_queue.borrow_mut().push_back(pos)
    }

    pub fn remove_marked_chunks(&mut self) {
        let removal_queue = self.removal_queue.get_mut();
        while let Some(pos) = removal_queue.pop_front() {
            self.mesh_cache.delete_chunk(pos);
        }
    }

    pub fn remove_chunk(&mut self, pos: IVec2) {
        self.mesh_cache.delete_chunk(pos);
    }

    pub fn clear(&mut self) {
        self.mesh_cache.clear();
    }
}
