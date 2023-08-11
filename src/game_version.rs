use std::fs::{self, DirEntry};
use std::path::PathBuf;

use rustc_hash::FxHashMap as HashMap;
use ultraviolet::Vec2;
use crate::block::Block;
use crate::block::properties::PropertyDefinition;
use crate::client::models::model::{BakedModel, VoxelModel};
use crate::client::textures::TextureObject;
use crate::client::textures::TextureObject::AtlasTexture;
use crate::minecraft::blocks;
use crate::minecraft::filetypes::{MCAtlasTextureFile, UniformAtlasTextureType, MCModelFile, MCBlockstateType};
use crate::minecraft::identifier::Identifier;
use crate::minecraft::registry::Registry;

pub enum GameVersion {
    B173,
    Orange,
}

impl GameVersion {
    pub fn load_registry(&self, registry: &mut Registry) {
        match self {
            Self::B173 => load_resources(registry),
            _ => {},
        }
    }
}

fn get_uv_from_atlas_index(texture_index: usize) -> [Vec2; 2] {
    let (u, v) = ((texture_index % 16) as f32 * 16., (texture_index / 16) as f32 * 16.,);
    let (u, v) = ([u, v], [u + 16., v + 16.]);
    const INV_ATLAS_SIZE: f32 = 1.0 / 256.;
    [Vec2::new((u[0] * INV_ATLAS_SIZE) as f32, (u[1] * INV_ATLAS_SIZE) as f32), Vec2::new((v[0] * INV_ATLAS_SIZE) as f32, (v[1] * INV_ATLAS_SIZE) as f32)]
}

fn make_atlas_tex(texture_index: usize) -> TextureObject {
    AtlasTexture { internal_uv: get_uv_from_atlas_index(texture_index) }
}



/** Apply a function to all files in dir and subdirs   
Will crash if depth is greater than number of allowed open files per program 
*/
fn iter_files_recursive<F: FnMut(&DirEntry)>(path: PathBuf, file_funct: &mut F) {
    if !path.is_dir() {
        log::error!("Not a dir: {}", path.display());
        return;
    }

    for f in fs::read_dir(path).unwrap() {
        let entry = f.unwrap();
        let entry_path = entry.path();
        if entry_path.is_dir() {
            iter_files_recursive(entry_path, file_funct);
        } else {
            file_funct(&entry);
        }
    }
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
fn make_model(registry: &Registry, identifier: &Identifier, model_files: &HashMap<Identifier, MCModelFile>, voxel_models: &mut HashMap<Identifier, VoxelModel>) -> Option<VoxelModel> {
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

pub fn load_resources(registry: &mut Registry) {

    let mut model_files = HashMap::default();
    let mut voxel_models = HashMap::default();
    let mut blockstate_files = HashMap::default();

    let namespace = "minecraft";
    let context = "models";
    let assets_dir = "../orange-mc-assets";

    let read_dir = format!("{}/assets/{}/{}/", assets_dir, namespace, context);
    iter_files_recursive(read_dir.clone().into(), &mut |entry| {
        if entry.file_name().to_str().unwrap().to_owned().ends_with(".json") {
            match serde_json::from_str::<MCModelFile>(fs::read_to_string(entry.path()).unwrap().as_str()) {
                Ok(model_file) => {
                    let resource_extension = entry.path().extension().map(|ext| format!(".{}", ext.to_string_lossy().to_string())).unwrap_or("".to_string());
                    let resource_path = entry.path().to_string_lossy().replace(&read_dir, "").replace(&resource_extension, "").replace("\\", "/");
                    let resource_id = Identifier::new(namespace.to_string(), resource_path.clone());
                    model_files.insert(resource_id, model_file);
                },
                Err(e) => { log::error!("Error processing {}: {}", entry.file_name().to_string_lossy(), e) }
            };
        } else {
            log::error!("Unknown File: {}", entry.path().display());
        }
    });

    let context = "blockstates";
    let read_dir = format!("{}/assets/{}/{}/", assets_dir, namespace, context);
    iter_files_recursive(read_dir.clone().into(), &mut |entry| {
        if entry.file_name().to_string_lossy().ends_with(".json") {
            match serde_json::from_str::<MCBlockstateType>(fs::read_to_string(entry.path()).unwrap().as_str()) {
                Ok(blockstate_file) => {
                    let resource_extension = entry.path().extension().map(|ext| format!(".{}", ext.to_string_lossy().to_string())).unwrap_or("".to_string());
                    let resource_path = entry.path().to_string_lossy().replace(&read_dir, "").replace(&resource_extension, "").replace("\\", "/");
                    let resource_id = Identifier::new(namespace.to_string(), resource_path);
                    blockstate_files.insert(resource_id, blockstate_file);
                },
                Err(e) => { log::error!("Error processing {}: {}", entry.file_name().to_string_lossy(), e) }
            };
        }
    });

    let atlas_texture_json_str = fs::read_to_string([assets_dir, "assets/minecraft/textures/block/terrain.mcatlas"].join("/"))
        .expect("Should have been able to read the file");
    let atlas_textures: MCAtlasTextureFile = serde_json::from_str(atlas_texture_json_str.as_str()).unwrap();

    {
        let textures = registry.get_texture_register_mut();
        for UniformAtlasTextureType { identifier, cell } in atlas_textures.atlas.get_uniform_textures() {
            let tex = make_atlas_tex(cell as usize);
            textures.insert(Identifier::from_str(identifier.as_str()), tex);
        }
    }

    for model_file_id in model_files.keys() {
        let voxel_model = make_model(registry, &model_file_id, &model_files, &mut voxel_models);
        if let Some(voxel_model) = voxel_model { voxel_models.insert(model_file_id.clone(), voxel_model); } else { log::warn!("Couln't make voxel model for {}", model_file_id); }
    }
    

    // Blocks & Items
    

    let missing_model_file = voxel_models.get(&Identifier::from_str("minecraft:block/missing")).expect("Did not have a missing model");

    let mut mapped_models: Vec<(Identifier, BakedModel)> = vec![];

    let textures = registry.get_texture_register();
    for state in registry.get_blockstate_register().get_elements() {
        let identifier = state.get_state_identifier();
        let block_id = state.get_block_identifier();

        let a = blockstate_files.get(block_id)
            .ok_or(crate::client::models::BlockstateParseError::NoBlockstateFile)
            .and_then(|blockstate_file| {
                crate::client::models::generate_blockstate_model(&blockstate_file, identifier.get_identifier_string(), &voxel_models, textures)
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
