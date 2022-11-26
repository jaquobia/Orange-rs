use std::collections::HashMap;
use std::path::PathBuf;

use image::{DynamicImage, Rgb32FImage, GenericImageView, EncodableLayout};
use image::io::Reader as ImageReader;

use crate::rendering::{Client};
use crate::rendering::{textures::TexWrapper};
use crate::{MC_HOME, mc_constants};


pub type TexMapType = HashMap<String, TexWrapper>;

pub fn mc_terrain_tex_layout(client: &mut Client) {
    let label = "mc_terrain_tex_layout";
    let layout = client.gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some(label),
    });
    client.layouts.insert(String::from(label), layout);
}

pub fn load_resources(client: &mut Client) {
    let gpu = &client.gpu;
    let tex_map = &mut client.textures;
    let resource_dir = MC_HOME.join("legacy_resources");
    let texture_layout = &client.layouts["mc_terrain_tex_layout"];
    for file_path in mc_constants::VEC_ASSETS2 {

        if file_path.ends_with(".txt") { continue }; // Only handle images

        let dir = resource_dir.join(file_path);
        let tex = load_mc_tex(&dir);
        let dims = tex.dimensions();
        let tex_dims = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = gpu.device.create_texture(
            &wgpu::TextureDescriptor {
                size: tex_dims,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
            }
        );
        gpu.queue.write_texture(
            diffuse_texture.as_image_copy(),
            tex.to_rgba8().as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dims.0),
                rows_per_image: std::num::NonZeroU32::new(dims.1),
            },
            tex_dims);
        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        tex_map.insert(String::from(file_path), TexWrapper::new(diffuse_texture, dims.into(), diffuse_texture_view, diffuse_sampler, &gpu.device, &texture_layout));
    }
}


fn create_missing_tex(_a: image::ImageError) -> DynamicImage {
    let mut rgb_tex = Rgb32FImage::new(2, 2);
    let pink_pixel = image::Rgb::<f32>([1.0, 1.0, 1.0]);
    let black_pixel = image::Rgb::<f32>([0.0, 0.0, 0.0]);
    rgb_tex.put_pixel(0, 0, pink_pixel);
    rgb_tex.put_pixel(0, 1, black_pixel);
    rgb_tex.put_pixel(1, 0, black_pixel);
    rgb_tex.put_pixel(1, 1, pink_pixel);
    DynamicImage::ImageRgb32F(rgb_tex)
}


pub fn load_mc_tex(dir: &PathBuf) -> DynamicImage {
    ImageReader::open(dir).unwrap().decode().unwrap_or_else(create_missing_tex)
}
