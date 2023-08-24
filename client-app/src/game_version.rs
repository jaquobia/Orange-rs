use std::path::PathBuf;

use crunch::Rotation;
use image::{GenericImage, GenericImageView};
use crate::rendering::textures::DiffuseTextureWrapper;
use rustc_hash::FxHashMap as HashMap;
use ultraviolet::Vec2;
use orange_rs::{block::Block, sprites::Sprite, models::{BlockstateParseError, self, generate_blockstate_model, model::{VoxelModel, BakedModel}}};
use orange_rs::block::properties::PropertyDefinition;
use orange_rs::minecraft::asset_loader::AssetLoader;
use orange_rs::minecraft::{blocks, asset_loader};
use orange_rs::minecraft::filetypes::{MCModel, MCAtlasSource};
use orange_rs::minecraft::identifier::Identifier;
use orange_rs::minecraft::registry::Registry;
use orange_rs::resource_loader;

use crate::game_client::Client;
use crate::mc_resource_handler::{ATLAS_TEXTURE_NAME, ATLAS_LAYOUT_NAME};

fn get_uv_from_atlas_index(texture_index: usize) -> [Vec2; 2] {
    let (u, v) = ((texture_index % 16) as f32 * 16., (texture_index / 16) as f32 * 16.,);
    let (u, v) = ([u, v], [u + 16., v + 16.]);
    const INV_ATLAS_SIZE: f32 = 1.0 / 256.;
    [Vec2::new((u[0] * INV_ATLAS_SIZE) as f32, (u[1] * INV_ATLAS_SIZE) as f32), Vec2::new((v[0] * INV_ATLAS_SIZE) as f32, (v[1] * INV_ATLAS_SIZE) as f32)]
}

fn make_atlas_tex(texture_index: usize) -> Sprite {
    let [uv_min, uv_max] = get_uv_from_atlas_index(texture_index);
    Sprite { uv_min, uv_max, parent_texture: Identifier::from_str("game") }
}

fn register_properties(registry: &mut Registry) {

    let properties = registry.get_property_register_mut();
    let property_list: &[(&str, &[&str])] = &[
        ("minecraft:boolean", &["false", "true"]),
        ("minecraft:block_half", &["bottom", "top"]),
        ("minecraft:redstone_side", &["side", "up"]),
        ("minecraft:orientation_2d", &["NS", "EW"]),
        ("minecraft:count_1", &["0", "1"]),
        ("minecraft:count_2", &["0", "1", "2"]),
        ("minecraft:count_3", &["0", "1", "2", "3"]),
        ("minecraft:count_4", &["0", "1", "2", "3", "4"]),
        ("minecraft:count_5", &["0", "1", "2", "3", "4", "5"]),
        ("minecraft:count_6", &["0", "1", "2", "3", "4", "5", "6"]),
        ("minecraft:count_7", &["0", "1", "2", "3", "4", "5", "6", "7"]),
        ("minecraft:count_15", &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"]),
        ("minecraft:facing", &["north", "south", "east", "west", "up", "down"]),
        ("minecraft:facing_horizontal", &["north", "south", "east", "west"]),
        ("minecraft:bed_part", &["foot", "head"]),
        ("minecraft:tall_grass_type", &["grass", "fern", "dead_bush"]),
        ("minecraft:tree_type", &["oak", "spruce", "birch"]),
        ("minecraft:slab_type", &["stone", "sandstone", "plank", "cobblestone"]),
        ("minecraft:rail_no_curve", &["north_south", "east_west", "ascending_north", "ascending_south", "ascending_east", "ascending_west"]),
        ("minecraft:rail_with_curve", &["north_south", "east_west", "ascending_north", "ascending_south", "ascending_east", "ascending_west", "north_east", "north_west", "south_east", "south_west"]),
        ("minecraft:color", &["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]),
    ];
    for property_def in property_list {
        properties.insert(PropertyDefinition::new(property_def.0.into(), property_def.1));
    }
}

fn register_blocks(registry: &mut Registry) {

    for block in blocks::blocks() {
        let block_id = registry.get_block_register_mut().insert(block);
        let block = registry.get_block_register_mut().get_element_from_index(block_id).unwrap();
        for state in Block::map_states(block, registry) {
            registry.get_blockstate_register_mut().insert_pointer(state);
        }
    }
    log::warn!("There are {} blocks", registry.get_block_register().get_elements().len());
    log::warn!("There are {} blockstates", registry.get_blockstate_register().get_elements().len());

}

pub fn register_content(registry: &mut Registry) {
    register_properties(registry);
    register_blocks(registry);
}

// TODO: Check for infinite recursion through already visited models
fn make_model(registry: &Registry, identifier: &Identifier, model_files: &HashMap<Identifier, MCModel>, voxel_models: &mut HashMap<Identifier, VoxelModel>) -> Option<VoxelModel> {
    let already_visited = false;
    if already_visited { return None; }

    if !model_files.contains_key(identifier) {
        log::warn!("No model file for {}", identifier);
        return None;
    }
    model_files.get(identifier).and_then(|model_file| {
        let mut model = match model_file.get_parent() {
            Some(parent) => {
                // Apply models ontop of parent
                let parent_id: Identifier = parent.into();
                let parent_model = if voxel_models.contains_key(&parent_id) {
                    voxel_models.get(&parent_id)
                } else {
                    make_model(registry, &parent_id, model_files, voxel_models).and_then(|model| {
                        voxel_models.insert(parent_id.clone(), model);
                        voxel_models.get(&parent_id)
                    })
                };
                parent_model.and_then(|parent| {
                    let mut model = VoxelModel::from_template(parent);
                    if model_file.elements().len() > 0 {
                        model.clear_elements(); // override parent elements
                    }
                    
                    Some(model)
                })
            }, // end has a parent
            None => {
                // Create model file as-is
                let mut model = VoxelModel::new();
                model.with_ambient_occlusion_nc(model_file.get_ambient_occlusion());
                Some(model)
            } // end has no parent
        }; // end match parent

        if let Some(model) = &mut model {
            for (texture_var, texture_id) in model_file.textures() {
                model.with_texture_nc(texture_var, texture_id);
            }
            for element in model_file.elements() {
                let voxel_element = element.to_voxel_element();
                model.with_element_nc(voxel_element);
            }
        }

        model
    })
}

pub fn load_resources(assets_directory: &PathBuf, default_resources: Option<PathBuf>) -> AssetLoader {
    let default_resourcepack = default_resources.unwrap_or_else(|| assets_directory.join("b173.zip"));
    let resource_locations = vec![default_resourcepack];
    let mut resource_loader = resource_loader::ResourceLoader::new();
    resource_loader.set_sources(&resource_locations);

    let mut asset_loader = asset_loader::AssetLoader::new();
    asset_loader.preload("b173", assets_directory);
    resource_loader.reload_system(&mut asset_loader);

    asset_loader
}

pub fn bake_resources(registry: &mut Registry, client: &mut Client, asset_loader: &AssetLoader, device: &wgpu::Device, queue: &wgpu::Queue) {


    let model_files = asset_loader.models();
    let blockstate_files = asset_loader.blockstates();

    #[derive(Clone, Debug)]
    struct Thingy {
        pub source_id: Identifier,
        pub target_id: Identifier,
        pub start_x: usize,
        pub start_y: usize,
    }

    log::warn!("Does have missing sprite? {:?}", asset_loader.sprites().get(&Identifier::from_str("minecraft:block/missing")));
    let mut items = vec![];
    if let Some(atlas_source) = asset_loader.atlases().get(&Identifier::from_str("game")) {
        for source in &atlas_source.sources {
            match source {
                MCAtlasSource::Directory { source, prefix } => {},
                MCAtlasSource::Single { resource, sprite } => {},
                MCAtlasSource::Filter { namespace, path } => {},
                MCAtlasSource::Unstitch { resource, divisor_x, divisor_y, regions } => {
                    let resource = if resource.starts_with("/") {
                        resource.strip_prefix("/").unwrap()
                    } else {
                        resource
                    };
                    let source_id = Identifier::from_str(resource);
                    for region in regions {
                        let target_id =  Identifier::from_str(&region.sprite);
                        let start_x = (region.x * divisor_x) as usize;
                        let start_y = (region.y * divisor_y) as usize;
                        let width = (region.width * divisor_x) as usize;
                        let height = (region.height * divisor_y) as usize;
                        let item = crunch::Item::new(Thingy { source_id: source_id.clone(), target_id, start_x, start_y }, width, height, Rotation::None);
                        items.push(item);
                    }
                }
            }
        }
    }
    items.push(crunch::Item::new(Thingy { source_id: Identifier::from_str("block/missing"), target_id: Identifier::from_str("block/missing"), start_x: 0, start_y: 0 }, 2, 2, Rotation::None));
    let result = crunch::pack_into_po2(4096, items).expect("Couldnt Pack Properly");
    let mut game_texture = image::RgbaImage::new(result.w.try_into().unwrap(), result.h.try_into().unwrap());
    let atlas_width = game_texture.width();
    let atlas_height = game_texture.height();
    // log::warn!("Texture Packing Result: {:?}", result.items);
    // log::warn!("Packed Items: {}x{}", atlas_width, atlas_height);
    for item in result.items {
        let sprite_width = item.rect.w as u32;
        let sprite_height = item.rect.h as u32;
        let sprite_atlas_x = item.rect.x as u32;
        let sprite_atlas_y = item.rect.y as u32;
        let sprite_image_x = item.data.start_x as u32;
        let sprite_image_y = item.data.start_y as u32;
        // log::warn!("Item {} -> {}: {:?}", item.data.source_id, item.data.target_id, item.rect);
        let other = asset_loader.sprites().get(&item.data.source_id).expect("Couldnt get source texture")
            .view(sprite_image_x, sprite_image_y, sprite_width, sprite_height);
        game_texture.copy_from(&*other, item.rect.x as u32, item.rect.y as u32);
        let uv_min = Vec2::new((sprite_atlas_x as f32) / (atlas_width as f32), (sprite_atlas_y as f32) / (atlas_height as f32));
        let uv_max = Vec2::new(((sprite_atlas_x + sprite_width) as f32) / (atlas_width as f32), ((sprite_atlas_y + sprite_height) as f32) / (atlas_height as f32));
        registry.get_sprite_register_mut().insert(item.data.target_id.clone(), Sprite { uv_min, uv_max, parent_texture: item.data.source_id.clone() });
    }


    let tex_dims = wgpu::Extent3d {
        width: atlas_width,
        height: atlas_height,
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
        game_texture.as_raw(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * atlas_width),
            rows_per_image: Some(atlas_height),
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
        (atlas_width, atlas_height).into(),
        diffuse_texture_view,
        sampler,
        device,
        client.get_layout(ATLAS_LAYOUT_NAME).unwrap(),
        );
    client.insert_texture(ATLAS_TEXTURE_NAME, texture);

    let mut voxel_models = HashMap::default();
    for model_file_id in model_files.keys() {
        let voxel_model = make_model(registry, &model_file_id, &model_files, &mut voxel_models);
        if let Some(voxel_model) = voxel_model { voxel_models.insert(model_file_id.clone(), voxel_model); } else { log::warn!("Couln't make voxel model for {}", model_file_id); }
    }
    

    // Blocks & Items
    

    let missing_model_file = voxel_models.get(&Identifier::from_str("minecraft:block/missing")).expect("Did not have a missing model");

    let mut mapped_models: Vec<(Identifier, BakedModel)> = vec![];

    let textures = registry.get_sprite_register();
    for state in registry.get_blockstate_register().get_elements() {
        let identifier = state.get_state_identifier();
        let block_id = state.get_block_identifier();

        let a = blockstate_files.get(block_id)
            .ok_or(BlockstateParseError::NoBlockstateFile)
            .and_then(|blockstate_file| {
                generate_blockstate_model(&blockstate_file, identifier.get_identifier_string(), &voxel_models, textures)
            });
        let blockstate_model = match a {
            Ok(model) => model,
            Err(e) => {
                log::warn!("Failed to create model for blockstate {}, reason: {}", identifier, e);
                missing_model_file.clone().bake(textures)
            }
        };
        mapped_models.push((state.get_state_identifier().clone(), blockstate_model));
    }

    for mapped_model in mapped_models {
        registry.get_model_register_mut().insert(mapped_model.0, mapped_model.1);
    }
}
