use std::collections::HashMap;
use std::path::PathBuf;

use image::{DynamicImage, Rgb32FImage, GenericImageView, EncodableLayout};
use image::io::Reader as ImageReader;
use wgpu::{Texture};

use crate::rendering::{WgpuData};
use crate::{MC_HOME, mc_constants};

pub struct TexWrapper(Texture, (u32, u32));

pub type TexMapType = HashMap<String, TexWrapper>;

pub fn load_resources(gpu: &WgpuData) -> TexMapType {
    let mut tex_map: TexMapType = HashMap::new();
    let resource_dir = MC_HOME.join("legacy_resources");
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
        tex_map.insert(String::from(file_path), TexWrapper(diffuse_texture, dims));
    }
    tex_map
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
