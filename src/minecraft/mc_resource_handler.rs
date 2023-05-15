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

pub fn create_resources(client: &mut Client) {

    generate_atlas_texture_bind_group_layout(client);
    generate_lightmap_texture_bind_group_layout(client);
    generate_camera_bind_group_layout(client);

    generate_camera_buffer(client);
    generate_camera_bind_group(client);

    generate_terrain_opaque_pipeline(client);
    generate_terrain_transparent_pipeline(client);

    generate_lightmap_texture(client);
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

pub fn generate_atlas_texture_bind_group_layout(client: &mut Client) {
    let layout = generate_basic_2d_texture_bind_group_layout(ATLAS_LAYOUT_NAME, client.get_device());
    client.insert_layout(ATLAS_LAYOUT_NAME, layout);
}

pub fn generate_lightmap_texture_bind_group_layout(client: &mut Client) {
    let layout = generate_basic_2d_texture_bind_group_layout(LIGHTMAP_LAYOUT_NAME, client.get_device());
    client.insert_layout(LIGHTMAP_LAYOUT_NAME, layout);
}

pub fn generate_camera_bind_group_layout(client: &mut Client) {
    let layout = client.get_device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

pub fn generate_camera_buffer(client: &mut Client) {
    let camera_buffer = client.get_device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(CAMERA_BUFFER_NAME),
        contents: bytemuck::cast_slice(&[client.projection.calc_matrix() * client.camera.calc_matrix()]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    client.insert_buffer(CAMERA_BUFFER_NAME, camera_buffer);
}

pub fn generate_camera_bind_group(client: &mut Client) {
    let bind_group = client.get_device().create_bind_group(&wgpu::BindGroupDescriptor {
        layout: client.get_layout(CAMERA_LAYOUT_NAME).unwrap(),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: client.get_buffer(CAMERA_BUFFER_NAME).unwrap().as_entire_binding(),
        }],
        label: Some(CAMERA_BIND_GROUP_NAME),
    });
    client.insert_bind_group(CAMERA_BIND_GROUP_NAME, bind_group);
}

pub fn generate_terrain_opaque_pipeline(client: &mut Client) {
    let device = client.get_device();

    let mut shader_data = String::new();
    let mut shader_file = File::open("./assets/shaders/shader.wgsl").unwrap();
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
                format: client.get_surface_configuration().format,
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

pub fn generate_terrain_transparent_pipeline(client: &mut Client) {

    let device = client.get_device();

    let mut shader_data = String::new();
    let mut shader_file = File::open("./assets/shaders/shader_transparent.wgsl").unwrap();
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
                format: client.get_surface_configuration().format,
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

pub fn generate_lightmap_texture(client: &mut Client) {
    let mut rgb_tex = Rgb32FImage::new(16, 16);

    for x in 0..16 {
        for y in 0..16 {
            rgb_tex.put_pixel(x, y, image::Rgb::<f32>([1.0, 1.0, 1.0]));
        }
    }

    let tex = DynamicImage::ImageRgb32F(rgb_tex);

    let dims = tex.dimensions();
    let width = dims.0;
    let height = dims.1;

    let tex_dims = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let tex_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let diffuse_texture = client.get_device().create_texture(&wgpu::TextureDescriptor {
        size: tex_dims,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: tex_format,
        view_formats: &[tex_format],
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
    });
    client.get_queue().write_texture(
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

    let sampler = client.get_device().create_sampler(&wgpu::SamplerDescriptor {
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
        client.get_device(),
        client.get_layout(LIGHTMAP_LAYOUT_NAME).unwrap(),
    );

    client.insert_texture(LIGHTMAP_TEXTURE_NAME, texture);
}

pub fn load_binary_resources(client: &mut Client) {
    for (path, _name, bytes) in DEFAULT_RESOURCES {
        if path.ends_with(".txt") || path.ends_with(".lang") {
            continue;
        }
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

        let diffuse_texture = client.get_device().create_texture(&wgpu::TextureDescriptor {
            size: tex_dims,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: tex_format,
            view_formats: &[tex_format],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });
        client.get_queue().write_texture(
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

        let sampler = client.get_device().create_sampler(&wgpu::SamplerDescriptor {
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
            client.get_device(),
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

pub const DEFAULT_RESOURCES: [(&str, &str, &[u8]); 83] = [
    (
        "terrain.png",
        "terrain",
        include_bytes!("../../resources/terrain.png"),
    ),
    (
        "pack.png",
        "pack",
        include_bytes!("../../resources/pack.png"),
    ),
    (
        "font.txt",
        "font",
        include_bytes!("../../resources/font.txt"),
    ),
    (
        "pack.txt",
        "pack",
        include_bytes!("../../resources/pack.txt"),
    ),
    (
        "particles.png",
        "particles",
        include_bytes!("../../resources/particles.png"),
    ),
    (
        "achievement/bg.png",
        "bg",
        include_bytes!("../../resources/achievement/bg.png"),
    ),
    (
        "achievement/icons.png",
        "icons",
        include_bytes!("../../resources/achievement/icons.png"),
    ),
    (
        "achievement/map.txt",
        "map",
        include_bytes!("../../resources/achievement/map.txt"),
    ),
    (
        "armor/chain_1.png",
        "chain_1",
        include_bytes!("../../resources/armor/chain_1.png"),
    ),
    (
        "armor/chain_2.png",
        "chain_2",
        include_bytes!("../../resources/armor/chain_2.png"),
    ),
    (
        "armor/cloth_1.png",
        "cloth_1",
        include_bytes!("../../resources/armor/cloth_1.png"),
    ),
    (
        "armor/cloth_2.png",
        "cloth_2",
        include_bytes!("../../resources/armor/cloth_2.png"),
    ),
    (
        "armor/diamond_1.png",
        "diamond_1",
        include_bytes!("../../resources/armor/diamond_1.png"),
    ),
    (
        "armor/diamond_2.png",
        "diamond_2",
        include_bytes!("../../resources/armor/diamond_2.png"),
    ),
    (
        "armor/gold_1.png",
        "gold_1",
        include_bytes!("../../resources/armor/gold_1.png"),
    ),
    (
        "armor/gold_2.png",
        "gold_2",
        include_bytes!("../../resources/armor/gold_2.png"),
    ),
    (
        "armor/iron_1.png",
        "iron_1",
        include_bytes!("../../resources/armor/iron_1.png"),
    ),
    (
        "armor/iron_2.png",
        "iron_2",
        include_bytes!("../../resources/armor/iron_2.png"),
    ),
    (
        "armor/power.png",
        "power",
        include_bytes!("../../resources/armor/power.png"),
    ),
    (
        "art/kz.png",
        "kz",
        include_bytes!("../../resources/art/kz.png"),
    ),
    (
        "environment/clouds.png",
        "clouds",
        include_bytes!("../../resources/environment/clouds.png"),
    ),
    (
        "environment/rain.png",
        "rain",
        include_bytes!("../../resources/environment/rain.png"),
    ),
    (
        "environment/snow.png",
        "snow",
        include_bytes!("../../resources/environment/snow.png"),
    ),
    (
        "font/default.png",
        "default",
        include_bytes!("../../resources/font/default.png"),
    ),
    (
        "gui/background.png",
        "background",
        include_bytes!("../../resources/gui/background.png"),
    ),
    (
        "gui/container.png",
        "container",
        include_bytes!("../../resources/gui/container.png"),
    ),
    (
        "gui/crafting.png",
        "crafting",
        include_bytes!("../../resources/gui/crafting.png"),
    ),
    (
        "gui/furnace.png",
        "furnace",
        include_bytes!("../../resources/gui/furnace.png"),
    ),
    (
        "gui/gui.png",
        "gui",
        include_bytes!("../../resources/gui/gui.png"),
    ),
    (
        "gui/icons.png",
        "icons",
        include_bytes!("../../resources/gui/icons.png"),
    ),
    (
        "gui/inventory.png",
        "inventory",
        include_bytes!("../../resources/gui/inventory.png"),
    ),
    (
        "gui/items.png",
        "items",
        include_bytes!("../../resources/gui/items.png"),
    ),
    (
        "gui/logo.png",
        "logo",
        include_bytes!("../../resources/gui/logo.png"),
    ),
    (
        "gui/particles.png",
        "particles",
        include_bytes!("../../resources/gui/particles.png"),
    ),
    (
        "gui/slot.png",
        "slot",
        include_bytes!("../../resources/gui/slot.png"),
    ),
    (
        "gui/trap.png",
        "trap",
        include_bytes!("../../resources/gui/trap.png"),
    ),
    (
        "gui/unknown_pack.png",
        "unknown_pack",
        include_bytes!("../../resources/gui/unknown_pack.png"),
    ),
    (
        "item/arrows.png",
        "arrows",
        include_bytes!("../../resources/item/arrows.png"),
    ),
    (
        "item/boat.png",
        "boat",
        include_bytes!("../../resources/item/boat.png"),
    ),
    (
        "item/cart.png",
        "cart",
        include_bytes!("../../resources/item/cart.png"),
    ),
    (
        "item/door.png",
        "door",
        include_bytes!("../../resources/item/door.png"),
    ),
    (
        "item/sign.png",
        "sign",
        include_bytes!("../../resources/item/sign.png"),
    ),
    (
        "lang/en_US.lang",
        "en_US",
        include_bytes!("../../resources/lang/en_US.lang"),
    ),
    (
        "lang/stats_US.lang",
        "stats_US",
        include_bytes!("../../resources/lang/stats_US.lang"),
    ),
    (
        "misc/dial.png",
        "dial",
        include_bytes!("../../resources/misc/dial.png"),
    ),
    (
        "misc/foliagecolor.png",
        "foliagecolor",
        include_bytes!("../../resources/misc/foliagecolor.png"),
    ),
    (
        "misc/footprint.png",
        "footprint",
        include_bytes!("../../resources/misc/footprint.png"),
    ),
    (
        "misc/grasscolor.png",
        "grasscolor",
        include_bytes!("../../resources/misc/grasscolor.png"),
    ),
    (
        "misc/mapbg.png",
        "mapbg",
        include_bytes!("../../resources/misc/mapbg.png"),
    ),
    (
        "misc/mapicons.png",
        "mapicons",
        include_bytes!("../../resources/misc/mapicons.png"),
    ),
    (
        "misc/pumpkinblur.png",
        "pumpkinblur",
        include_bytes!("../../resources/misc/pumpkinblur.png"),
    ),
    (
        "misc/shadow.png",
        "shadow",
        include_bytes!("../../resources/misc/shadow.png"),
    ),
    (
        "misc/vignette.png",
        "vignette",
        include_bytes!("../../resources/misc/vignette.png"),
    ),
    (
        "misc/water.png",
        "water",
        include_bytes!("../../resources/misc/water.png"),
    ),
    (
        "misc/watercolor.png",
        "watercolor",
        include_bytes!("../../resources/misc/watercolor.png"),
    ),
    (
        "mob/char.png",
        "char",
        include_bytes!("../../resources/mob/char.png"),
    ),
    (
        "mob/chicken.png",
        "chicken",
        include_bytes!("../../resources/mob/chicken.png"),
    ),
    (
        "mob/cow.png",
        "cow",
        include_bytes!("../../resources/mob/cow.png"),
    ),
    (
        "mob/creeper.png",
        "creeper",
        include_bytes!("../../resources/mob/creeper.png"),
    ),
    (
        "mob/ghast.png",
        "ghast",
        include_bytes!("../../resources/mob/ghast.png"),
    ),
    (
        "mob/ghast_fire.png",
        "ghast_fire",
        include_bytes!("../../resources/mob/ghast_fire.png"),
    ),
    (
        "mob/pig.png",
        "pig",
        include_bytes!("../../resources/mob/pig.png"),
    ),
    (
        "mob/pigman.png",
        "pigman",
        include_bytes!("../../resources/mob/pigman.png"),
    ),
    (
        "mob/pigzombie.png",
        "pigzombie",
        include_bytes!("../../resources/mob/pigzombie.png"),
    ),
    (
        "mob/saddle.png",
        "saddle",
        include_bytes!("../../resources/mob/saddle.png"),
    ),
    (
        "mob/sheep.png",
        "sheep",
        include_bytes!("../../resources/mob/sheep.png"),
    ),
    (
        "mob/sheep_fur.png",
        "sheep_fur",
        include_bytes!("../../resources/mob/sheep_fur.png"),
    ),
    (
        "mob/silverfish.png",
        "silverfish",
        include_bytes!("../../resources/mob/silverfish.png"),
    ),
    (
        "mob/skeleton.png",
        "skeleton",
        include_bytes!("../../resources/mob/skeleton.png"),
    ),
    (
        "mob/slime.png",
        "slime",
        include_bytes!("../../resources/mob/slime.png"),
    ),
    (
        "mob/spider.png",
        "spider",
        include_bytes!("../../resources/mob/spider.png"),
    ),
    (
        "mob/spider_eyes.png",
        "spider_eyes",
        include_bytes!("../../resources/mob/spider_eyes.png"),
    ),
    (
        "mob/squid.png",
        "squid",
        include_bytes!("../../resources/mob/squid.png"),
    ),
    (
        "mob/wolf.png",
        "wolf",
        include_bytes!("../../resources/mob/wolf.png"),
    ),
    (
        "mob/wolf_angry.png",
        "wolf_angry",
        include_bytes!("../../resources/mob/wolf_angry.png"),
    ),
    (
        "mob/wolf_tame.png",
        "wolf_tame",
        include_bytes!("../../resources/mob/wolf_tame.png"),
    ),
    (
        "mob/zombie.png",
        "zombie",
        include_bytes!("../../resources/mob/zombie.png"),
    ),
    (
        "terrain/moon.png",
        "moon",
        include_bytes!("../../resources/terrain/moon.png"),
    ),
    (
        "terrain/sun.png",
        "sun",
        include_bytes!("../../resources/terrain/sun.png"),
    ),
    (
        "title/black.png",
        "black",
        include_bytes!("../../resources/title/black.png"),
    ),
    (
        "title/mclogo.png",
        "mclogo",
        include_bytes!("../../resources/title/mclogo.png"),
    ),
    (
        "title/mojang.png",
        "mojang",
        include_bytes!("../../resources/title/mojang.png"),
    ),
    (
        "title/splashes.txt",
        "splashes",
        include_bytes!("../../resources/title/splashes.txt"),
    ),
];
