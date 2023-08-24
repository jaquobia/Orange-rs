use wgpu::{Buffer, RenderPass};

pub struct Mesh {
    pub opaque_vertex_buffer: Buffer,
    pub transparent_vertex_buffer: Buffer,
    pub opaque_index_buffer: Buffer,
    pub transparent_index_buffer: Buffer,
    pub num_vertices_opaque: u32,
    pub num_vertices_transparent: u32,
    pub num_indices_opaque: u32,
    pub num_indices_transparent: u32,
}

impl Mesh {
    pub fn new(
        opaque_vertex_buffer: Buffer,
        transparent_vertex_buffer: Buffer,
        num_vertices_opaque: u32,
        num_vertices_transparent: u32,
        opaque_index_buffer: Buffer,
        transparent_index_buffer: Buffer,
        num_indices_opaque: u32,
        num_indices_transparent: u32,
    ) -> Self {
        Self {
            opaque_vertex_buffer,
            transparent_vertex_buffer,
            num_vertices_opaque,
            num_vertices_transparent,
            opaque_index_buffer,
            transparent_index_buffer,
            num_indices_opaque,
            num_indices_transparent,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.opaque_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.opaque_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices_opaque, 0, 0..1);
    }
    pub fn draw_transparent<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.transparent_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.transparent_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices_transparent, 0, 0..1);
    }
}
