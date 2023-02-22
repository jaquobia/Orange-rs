use ultraviolet::UVec2;
use wgpu::{BindGroup, BindGroupLayout, Device, Sampler, Texture, TextureFormat, TextureView};

use super::wgpu_struct::WgpuData;

pub struct DiffuseTextureWrapper {
    texture: Texture,
    size: UVec2,
    view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup,
}

impl DiffuseTextureWrapper {
    pub fn new(
        texture: Texture,
        size: UVec2,
        view: TextureView,
        sampler: Sampler,
        device: &Device,
        layout: &BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        Self {
            texture,
            size,
            view,
            sampler,
            bind_group,
        }
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn get_size(&self) -> UVec2 {
        self.size
    }
}

// pub const DEPTH_FORMAT: wgpu::TextureFormat = ;

pub struct DepthTextureWrapper {
    texture: Texture,
    view: TextureView,
    size: UVec2,
    sampler: Sampler,
    texture_format: TextureFormat,
}

impl DepthTextureWrapper {
    pub fn new(gpu: &WgpuData, depth_tex_format: TextureFormat, label: &str) -> Self {
        let config = &gpu.config;
        let device = &gpu.device;
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_tex_format,
            view_formats: &[depth_tex_format],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        Self {
            texture,
            size: UVec2::new(config.width, config.height),
            view,
            sampler,
            texture_format: depth_tex_format,
        }
    }

    pub fn get_texture(&self) -> &Texture {
        &self.texture
    }

    pub fn get_view(&self) -> &TextureView {
        &self.view
    }

    pub fn get_sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn get_size(&self) -> UVec2 {
        self.size
    }

    pub fn get_texture_format(&self) -> TextureFormat {
        self.texture_format
    }
}
