use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout, GenericImageView, Rgb32FImage};
use wgpu::util::DeviceExt;

use crate::client::{rendering::textures::DiffuseTextureWrapper, Client};
use crate::client::rendering::verticies::TerrainVertex;

pub type TexMapType = HashMap<String, DiffuseTextureWrapper>;

pub static ATLAS_LAYOUT_NAME: &str = "atlas_layout";
pub static LIGHTMAP_LAYOUT_NAME: &str = "lightmap_layout";
pub static CAMERA_LAYOUT_NAME: &str = "camera_layout";

pub static CAMERA_BUFFER_NAME: &str = "camera_buffer";

pub static CAMERA_BIND_GROUP_NAME: &str = "camera_bind_group";

pub static TERRAIN_OPAQUE_PIPELINE: &str = "opaque_terrain_shader";
pub static TERRAIN_TRANSPARENT_PIPELINE: &str = "transparent_terrain_shader";

pub static ATLAS_TEXTURE_NAME: &str = "minecraft:atlas";
pub static LIGHTMAP_TEXTURE_NAME: &str = "minecraft:lightmap";

pub fn create_resources(client: &mut Client, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {

    generate_atlas_texture_bind_group_layout(client, device);
    generate_lightmap_texture_bind_group_layout(client, device);
    generate_camera_bind_group_layout(client, device);

    generate_camera_buffer(client, device);
    generate_camera_bind_group(client, device);

    generate_terrain_opaque_pipeline(client, device, config);
    generate_terrain_transparent_pipeline(client, device, config);

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

pub fn generate_terrain_opaque_pipeline(client: &mut Client, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {

    let mut shader_data = String::new();
    let mut shader_file = File::open("../orange-mc-assets/assets/shaders/shader.wgsl").unwrap();
    shader_file.read_to_string(&mut shader_data).unwrap();

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

pub fn generate_terrain_transparent_pipeline(client: &mut Client, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {

    let mut shader_data = String::new();
    let mut shader_file = File::open("../orange-mc-assets/assets/shaders/shader_transparent.wgsl").unwrap();
    shader_file.read_to_string(&mut shader_data).unwrap();

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

    tex.to_rgb16().save_with_format("../orange-mc-assets/assets/minecraft/textures/lightmap.png", image::ImageFormat::Png).unwrap();

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

pub fn load_binary_resources(client: &mut Client, device: &wgpu::Device, queue: &wgpu::Queue) {
    for (path, _name) in DEFAULT_RESOURCES {
        if path.ends_with(".txt") || path.ends_with(".lang") {
            continue;
        }
        let resource_dir = "./resources";
        let bytes = std::fs::read([resource_dir, path].join("/")).unwrap();
        let reader = image::io::Reader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .expect("Cursor io never fails");

        let image = reader.decode().unwrap();
        let dims = image.dimensions();
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
            image.to_rgba8().as_bytes(),
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
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = DiffuseTextureWrapper::new(
            diffuse_texture,
            dims.into(),
            diffuse_texture_view,
            sampler,
            device,
            client.get_layout(ATLAS_LAYOUT_NAME).unwrap(),
        );
        client.insert_texture(path, texture);
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
    ImageReader::open(dir)
        .unwrap()
        .decode()
        .unwrap_or_else(create_missing_tex)
}

pub const DEFAULT_RESOURCES: [(&str, &str); 83] = [
    (
        "terrain.png",
        "terrain",
    ),
    (
        "pack.png",
        "pack",
    ),
    (
        "font.txt",
        "font",
    ),
    (
        "pack.txt",
        "pack",
    ),
    (
        "particles.png",
        "particles",
    ),
    (
        "achievement/bg.png",
        "bg",
    ),
    (
        "achievement/icons.png",
        "icons",
    ),
    (
        "achievement/map.txt",
        "map",
    ),
    (
        "armor/chain_1.png",
        "chain_1",
    ),
    (
        "armor/chain_2.png",
        "chain_2",
    ),
    (
        "armor/cloth_1.png",
        "cloth_1",
    ),
    (
        "armor/cloth_2.png",
        "cloth_2",
    ),
    (
        "armor/diamond_1.png",
        "diamond_1",
    ),
    (
        "armor/diamond_2.png",
        "diamond_2",
    ),
    (
        "armor/gold_1.png",
        "gold_1",
    ),
    (
        "armor/gold_2.png",
        "gold_2",
    ),
    (
        "armor/iron_1.png",
        "iron_1",
    ),
    (
        "armor/iron_2.png",
        "iron_2",
    ),
    (
        "armor/power.png",
        "power",
    ),
    (
        "art/kz.png",
        "kz",
    ),
    (
        "environment/clouds.png",
        "clouds",
    ),
    (
        "environment/rain.png",
        "rain",
    ),
    (
        "environment/snow.png",
        "snow",
    ),
    (
        "font/default.png",
        "default",
    ),
    (
        "gui/background.png",
        "background",
    ),
    (
        "gui/container.png",
        "container",
    ),
    (
        "gui/crafting.png",
        "crafting",
    ),
    (
        "gui/furnace.png",
        "furnace",
    ),
    (
        "gui/gui.png",
        "gui",
    ),
    (
        "gui/icons.png",
        "icons",
    ),
    (
        "gui/inventory.png",
        "inventory",
    ),
    (
        "gui/items.png",
        "items",
    ),
    (
        "gui/logo.png",
        "logo",
    ),
    (
        "gui/particles.png",
        "particles",
    ),
    (
        "gui/slot.png",
        "slot",
    ),
    (
        "gui/trap.png",
        "trap",
    ),
    (
        "gui/unknown_pack.png",
        "unknown_pack",
    ),
    (
        "item/arrows.png",
        "arrows",
    ),
    (
        "item/boat.png",
        "boat",
    ),
    (
        "item/cart.png",
        "cart",
    ),
    (
        "item/door.png",
        "door",
    ),
    (
        "item/sign.png",
        "sign",
    ),
    (
        "lang/en_US.lang",
        "en_US",
    ),
    (
        "lang/stats_US.lang",
        "stats_US",
    ),
    (
        "misc/dial.png",
        "dial",
    ),
    (
        "misc/foliagecolor.png",
        "foliagecolor",
    ),
    (
        "misc/footprint.png",
        "footprint",
    ),
    (
        "misc/grasscolor.png",
        "grasscolor",
    ),
    (
        "misc/mapbg.png",
        "mapbg",
    ),
    (
        "misc/mapicons.png",
        "mapicons",
    ),
    (
        "misc/pumpkinblur.png",
        "pumpkinblur",
    ),
    (
        "misc/shadow.png",
        "shadow",
    ),
    (
        "misc/vignette.png",
        "vignette",
    ),
    (
        "misc/water.png",
        "water",
    ),
    (
        "misc/watercolor.png",
        "watercolor",
    ),
    (
        "mob/char.png",
        "char",
    ),
    (
        "mob/chicken.png",
        "chicken",
    ),
    (
        "mob/cow.png",
        "cow",
    ),
    (
        "mob/creeper.png",
        "creeper",
    ),
    (
        "mob/ghast.png",
        "ghast",
    ),
    (
        "mob/ghast_fire.png",
        "ghast_fire",
    ),
    (
        "mob/pig.png",
        "pig",
    ),
    (
        "mob/pigman.png",
        "pigman",
    ),
    (
        "mob/pigzombie.png",
        "pigzombie",
    ),
    (
        "mob/saddle.png",
        "saddle",
    ),
    (
        "mob/sheep.png",
        "sheep",
    ),
    (
        "mob/sheep_fur.png",
        "sheep_fur",
    ),
    (
        "mob/silverfish.png",
        "silverfish",
    ),
    (
        "mob/skeleton.png",
        "skeleton",
    ),
    (
        "mob/slime.png",
        "slime",
    ),
    (
        "mob/spider.png",
        "spider",
    ),
    (
        "mob/spider_eyes.png",
        "spider_eyes",
    ),
    (
        "mob/squid.png",
        "squid",
    ),
    (
        "mob/wolf.png",
        "wolf",
    ),
    (
        "mob/wolf_angry.png",
        "wolf_angry",
    ),
    (
        "mob/wolf_tame.png",
        "wolf_tame",
    ),
    (
        "mob/zombie.png",
        "zombie",
    ),
    (
        "terrain/moon.png",
        "moon",
    ),
    (
        "terrain/sun.png",
        "sun",
    ),
    (
        "title/black.png",
        "black",
    ),
    (
        "title/mclogo.png",
        "mclogo",
    ),
    (
        "title/mojang.png",
        "mojang",
    ),
    (
        "title/splashes.txt",
        "splashes",
    ),
];
