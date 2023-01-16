use wgpu::{Buffer, RenderPass};

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_verticies: u32,
    pub num_indicies: u32,
}

impl Mesh {
    pub fn new(
        vertex_buffer: Buffer,
        num_verticies: u32,
        index_buffer: Buffer,
        num_indicies: u32,
    ) -> Self {
        Self {
            vertex_buffer,
            num_verticies,
            index_buffer,
            num_indicies,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indicies, 0, 0..1);
    }
}
