use ultraviolet::UVec2;
use wgpu::{Texture, TextureView, Sampler, BindGroup, Device, BindGroupLayout};


pub struct TexWrapper{
    texture: Texture,
    size: UVec2,
    view: TextureView,
    sampler: Sampler,
    pub bind_group: BindGroup,
}

impl TexWrapper {
    pub fn new(texture: Texture, size: UVec2, view: TextureView, sampler: Sampler, device: &Device, layout: &BindGroupLayout) -> Self {
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );
        Self {
            texture,
            size,
            view,
            sampler,
            bind_group,
        }
    }

}
