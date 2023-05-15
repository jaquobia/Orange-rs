use ultraviolet::{Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    position: Vec3,
    color: Vec3, // u8 index -> texture; 2u8 2u8 -> uv
    normal: Vec3,
    texture: Vec2,
    ao: u32, // u8 ao, u16 block light (12 bits rgb 4-4-4, 4-lightmap)
}

impl TerrainVertex {
    pub fn new(
        position: Vec3,
        color: Vec3,
        normal: Vec3,
        texture: Vec2,
        ao: u32,
    ) -> Self {
        Self {
            position,
            color,
            normal,
            texture,
            ao,
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x3,
        3 => Float32x2,
        4 => Uint32,
        ];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
