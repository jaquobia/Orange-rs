use std::{collections::VecDeque, cell::RefCell};

use crate::world::chunk_map::ChunkMap;
use ultraviolet::IVec2;

use crate::client::client_chunk::ClientChunk;

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

    pub fn construct_chunk(&mut self, mut meshes: Vec<Mesh>, pos: IVec2) {
        let mut chunk = ClientChunk::new(pos, self.chunk_height as usize);
        for i in 0..meshes.len() {
            chunk.set_section(i, meshes.pop());
        }
        chunk.mark_meshed();
        chunk.unmark_for_meshing();
        self.mesh_cache.set_chunk(pos, Some(chunk));
    }

    pub fn construct_chunk_empty(&mut self, pos: IVec2) {
        let mut chunk = ClientChunk::new(pos, self.chunk_height as usize);
        for i in 0..self.chunk_height as usize {
            chunk.set_section(i, None);
        } 
        self.mesh_cache.set_chunk(pos, Some(chunk));
    }

    pub fn get_chunks(&self) -> &Vec<WorldRenderChunkType> {
        self.mesh_cache.chunks()
    }

    pub fn get_chunks_mut(&mut self) -> &mut Vec<WorldRenderChunkType> {
        self.mesh_cache.chunks_mut()
    }

    pub fn get_cache(&self) -> &ChunkMap<WorldRenderChunkType> {
        &self.mesh_cache
    }

    pub fn set_section_mesh(&mut self, mesh: Mesh, pos: IVec2, section: usize) {

        if let Some(chunk) = self.mesh_cache.get_chunk_pos_mut(pos.x, pos.y) {
           chunk.set_section(section, Some(mesh));
            return;
        } else {
            self.construct_chunk_empty(pos);
            if let Some(chunk) = self.mesh_cache.get_chunk_pos_mut(pos.x, pos.y) {
                chunk.set_section(section, Some(mesh));
                return;
            }
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
