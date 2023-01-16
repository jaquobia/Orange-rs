use std::collections::HashMap;
use std::path::PathBuf;

use image::io::Reader as ImageReader;
use image::{DynamicImage, EncodableLayout, GenericImageView, Rgb32FImage};

use crate::rendering::textures::DiffuseTextureWrapper;
use crate::Client;

pub type TexMapType = HashMap<String, DiffuseTextureWrapper>;

pub fn mc_terrain_tex_layout(client: &mut Client) {
    let label = "mc_terrain_tex_layout";
    let layout = client
        .gpu
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

pub fn load_binary_resources(client: &mut Client) {
    let gpu = &mut client.gpu;
    let tex_map = &mut client.textures;
    let texture_layout = &client.layouts["mc_terrain_tex_layout"];

    for (path, name, bytes) in DEFAULT_RESOURCES {
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
        let diffuse_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            size: tex_dims,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("diffuse_texture"),
        });
        gpu.queue.write_texture(
            diffuse_texture.as_image_copy(),
            image.to_rgba8().as_bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            tex_dims,
        );
        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        tex_map.insert(
            String::from(path),
            DiffuseTextureWrapper::new(
                diffuse_texture,
                dims.into(),
                diffuse_texture_view,
                gpu.device.create_sampler(&wgpu::SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    ..Default::default()
                }),
                &gpu.device,
                &texture_layout,
            ),
        );
    }
}

// pub fn load_resources(client: &mut Client) {
//     let gpu = &client.gpu;
//     let tex_map = &mut client.textures;
//     let resource_dir = MC_HOME.join("legacy_resources");
//     let texture_layout = &client.layouts["mc_terrain_tex_layout"];
//     for file_path in mc_constants::VEC_ASSETS2 {

//         if file_path.ends_with(".txt") { continue }; // Only handle images

//         let dir = resource_dir.join(file_path);
//         let tex = load_mc_tex(&dir);
//         let dims = tex.dimensions();
//         let tex_dims = wgpu::Extent3d {
//             width: dims.0,
//             height: dims.1,
//             depth_or_array_layers: 1,
//         };
//         let diffuse_texture = gpu.device.create_texture(
//             &wgpu::TextureDescriptor {
//                 size: tex_dims,
//                 mip_level_count: 1,
//                 sample_count: 1,
//                 dimension: wgpu::TextureDimension::D2,
//                 format: wgpu::TextureFormat::Rgba8UnormSrgb,
//                 usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//                 label: Some("diffuse_texture"),
//             }
//         );
//         gpu.queue.write_texture(
//             diffuse_texture.as_image_copy(),
//             tex.to_rgba8().as_bytes(),
//             wgpu::ImageDataLayout {
//                 offset: 0,
//                 bytes_per_row: std::num::NonZeroU32::new(4 * dims.0),
//                 rows_per_image: std::num::NonZeroU32::new(dims.1),
//             },
//             tex_dims);
//         let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
//         let diffuse_sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
//             address_mode_u: wgpu::AddressMode::ClampToEdge,
//             address_mode_v: wgpu::AddressMode::ClampToEdge,
//             address_mode_w: wgpu::AddressMode::ClampToEdge,
//             mag_filter: wgpu::FilterMode::Nearest,
//             min_filter: wgpu::FilterMode::Nearest,
//             mipmap_filter: wgpu::FilterMode::Nearest,
//             ..Default::default()
//         });
//         tex_map.insert(String::from(file_path), DiffuseTextureWrapper::new(diffuse_texture, dims.into(), diffuse_texture_view, diffuse_sampler, &gpu.device, &texture_layout));
//     }
// }

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
