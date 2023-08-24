use orange_rs::minecraft::asset_loader::AssetLoader;
use orange_rs::minecraft::identifier::Identifier;
use rustc_hash::FxHashMap as HashMap;

use image::{DynamicImage, EncodableLayout, GenericImageView, Rgb32FImage};
use wgpu::util::DeviceExt;

use crate::{rendering::{textures::DiffuseTextureWrapper, verticies::TerrainVertex}, game_client::Client};

pub type TexMapType = HashMap<String, DiffuseTextureWrapper>;

pub static ATLAS_LAYOUT_NAME: &str = "atlas_layout";
pub static LIGHTMAP_LAYOUT_NAME: &str = "lightmap_layout";
pub static CAMERA_LAYOUT_NAME: &str = "camera_layout";

pub static CAMERA_BUFFER_NAME: &str = "camera_buffer";

pub static CAMERA_BIND_GROUP_NAME: &str = "camera_bind_group";

pub static TERRAIN_OPAQUE_PIPELINE: &str = "shader";
pub static TERRAIN_TRANSPARENT_PIPELINE: &str = "shader_transparent";

pub static ATLAS_TEXTURE_NAME: &str = "minecraft:game";
pub static LIGHTMAP_TEXTURE_NAME: &str = "minecraft:lightmap";

pub fn create_resources(client: &mut Client, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, asset_loader: &AssetLoader) {

    generate_atlas_texture_bind_group_layout(client, device);
    generate_lightmap_texture_bind_group_layout(client, device);
    generate_camera_bind_group_layout(client, device);

    generate_camera_buffer(client, device);
    generate_camera_bind_group(client, device);

    generate_terrain_opaque_pipeline(client, device, config, asset_loader.shaders().get(&Identifier::from_str(TERRAIN_OPAQUE_PIPELINE)).expect("Did not have the terrain opaque shaders"));
    generate_terrain_transparent_pipeline(client, device, config, asset_loader.shaders().get(&Identifier::from_str(TERRAIN_TRANSPARENT_PIPELINE)).expect("Did not have the terrain opaque shaders"));

    generate_lightmap_texture(client, device, queue);
}

pub fn generate_basic_2d_texture_bind_group_layout<T: AsRef<str>>(label: T, device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: Some(label.as_ref()),
    })
}

pub fn generate_atlas_texture_bind_group_layout(client: &mut Client, device: &wgpu::Device) {
    let layout = generate_basic_2d_texture_bind_group_layout(ATLAS_LAYOUT_NAME, device);
    client.insert_layout(ATLAS_LAYOUT_NAME, layout);
}

pub fn generate_lightmap_texture_bind_group_layout(client: &mut Client, device: &wgpu::Device) {
    let layout = generate_basic_2d_texture_bind_group_layout(LIGHTMAP_LAYOUT_NAME, device);
    client.insert_layout(LIGHTMAP_LAYOUT_NAME, layout);
}

pub fn generate_camera_bind_group_layout(client: &mut Client, device: &wgpu::Device) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(CAMERA_LAYOUT_NAME),
    });

    client.insert_layout(CAMERA_LAYOUT_NAME, layout);
}

pub fn generate_camera_buffer(client: &mut Client, device: &wgpu::Device) {
    let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(CAMERA_BUFFER_NAME),
        contents: bytemuck::cast_slice(&[client.projection.calc_matrix() * client.camera.calc_matrix()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    client.insert_buffer(CAMERA_BUFFER_NAME, camera_buffer);
}

pub fn generate_camera_bind_group(client: &mut Client, device: &wgpu::Device) {
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: client.get_layout(CAMERA_LAYOUT_NAME).unwrap(),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: client.get_buffer(CAMERA_BUFFER_NAME).unwrap().as_entire_binding(),
        }],
        label: Some(CAMERA_BIND_GROUP_NAME),
    });
    client.insert_bind_group(CAMERA_BIND_GROUP_NAME, bind_group);
}

pub fn generate_terrain_opaque_pipeline(client: &mut Client, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, shader_data: &String) {

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(TERRAIN_OPAQUE_PIPELINE),
        source: wgpu::ShaderSource::Wgsl(shader_data.into()),
    });

    let camera_bind_group_layout = client.get_layout(CAMERA_LAYOUT_NAME).unwrap();
    let atlas_bind_group_layout = client.get_layout(ATLAS_LAYOUT_NAME).unwrap();
    let lightmap_bind_group_layout = client.get_layout(LIGHTMAP_LAYOUT_NAME).unwrap();

    let pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{}_layout", TERRAIN_OPAQUE_PIPELINE).as_str()),
            bind_group_layouts: &[camera_bind_group_layout, atlas_bind_group_layout, lightmap_bind_group_layout],
            push_constant_ranges: &[],
        });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(TERRAIN_OPAQUE_PIPELINE),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[TerrainVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    client.insert_pipeline(TERRAIN_OPAQUE_PIPELINE, pipeline);
}

pub fn generate_terrain_transparent_pipeline(client: &mut Client, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, shader_data: &String) {

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(TERRAIN_TRANSPARENT_PIPELINE),
        source: wgpu::ShaderSource::Wgsl(shader_data.into()),
    });

    let camera_bind_group_layout = client.get_layout(CAMERA_LAYOUT_NAME).unwrap();
    let atlas_bind_group_layout = client.get_layout(ATLAS_LAYOUT_NAME).unwrap();
    let lightmap_bind_group_layout = client.get_layout(LIGHTMAP_LAYOUT_NAME).unwrap();

    let pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{}_layout", TERRAIN_TRANSPARENT_PIPELINE).as_str()),
            bind_group_layouts: &[camera_bind_group_layout, atlas_bind_group_layout, lightmap_bind_group_layout],
            push_constant_ranges: &[],
        });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(TERRAIN_TRANSPARENT_PIPELINE),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[TerrainVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),

        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    client.insert_pipeline(TERRAIN_TRANSPARENT_PIPELINE, pipeline);
}

pub fn generate_lightmap_texture(client: &mut Client, device: &wgpu::Device, queue: &wgpu::Queue) {
    let width = 16;
    let height = 1;
    let widthpheight = (width - 1) + (height - 1);
    let mut rgb_tex = Rgb32FImage::new(width, height);

    for x in 0..width {
        for y in 0..height {
            let xf: f32 = x as f32;
            let r = xf / 15.0;
            let px = r / ((3.0 - 3.0 * r) + 1.0) * 0.95 + 0.05;
            rgb_tex.put_pixel(x, y, image::Rgb::<f32>([px, px, px]));
        }
    }



    let tex = DynamicImage::ImageRgb32F(rgb_tex);

    // tex.to_rgb16().save_with_format("../orange-mc-assets/assets/minecraft/textures/lightmap.png", image::ImageFormat::Png).unwrap();

    let dims = tex.dimensions();
    let width = dims.0;
    let height = dims.1;

    let tex_dims = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let tex_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: tex_dims,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: tex_format,
        view_formats: &[tex_format],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
    });
    queue.write_texture(
        diffuse_texture.as_image_copy(),
        tex.to_rgba8().as_bytes(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        tex_dims,
    );
    let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let texture = DiffuseTextureWrapper::new(
        diffuse_texture,
        dims.into(),
        diffuse_texture_view,
        sampler,
        device,
        client.get_layout(LIGHTMAP_LAYOUT_NAME).unwrap(),
    );

    client.insert_texture(LIGHTMAP_TEXTURE_NAME, texture);
}
