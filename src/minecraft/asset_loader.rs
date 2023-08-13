use std::path::PathBuf;

use image::DynamicImage;
use rustc_hash::FxHashMap as HashMap;

use crate::resource_loader::{ResourceSystem, ResourceCategory};

use super::{filetypes::{MCModelFile, MCBlockstateType}, identifier::Identifier};

pub struct AssetLoader {
    domain: String,
    categories: Vec<ResourceCategory>,
    model_files: HashMap<Identifier, MCModelFile>,
    blockstate_files: HashMap<Identifier, MCBlockstateType>,
    sprites: HashMap<Identifier, DynamicImage>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            domain: String::from("assets"),
            categories: vec![
                ResourceCategory::new("textures", vec!["png", "mcmeta", "mcatlas"]),
                ResourceCategory::new("models", vec!["json"]),
                ResourceCategory::new("blockstates", vec!["json"]),
            ],
            model_files: HashMap::default(),
            blockstate_files: HashMap::default(),
            sprites: HashMap::default(),
        }
    }

    pub fn preload<S: AsRef<str>>(&mut self, preload_file_name: S, assets_directory: &PathBuf) {
        self.inner_preload(preload_file_name.as_ref(), assets_directory);
    }

    pub fn models(&self) -> &HashMap<Identifier, MCModelFile> {
        &self.model_files
    }
    
    pub fn blockstates(&self) -> &HashMap<Identifier, MCBlockstateType> {
        &self.blockstate_files
    }

    pub fn sprites(&self) -> &HashMap<Identifier, DynamicImage> {
        &self.sprites
    }

    fn try_load_model(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(model) = serde_json::from_slice(contents) {
            self.model_files.insert(Identifier::new(namespace, file_name), model);
        }
    }

    fn try_load_blockstate(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(blockstate) = serde_json::from_slice(contents) {
            self.blockstate_files.insert(Identifier::new(namespace, file_name), blockstate);
        }
    }

    fn inner_preload(&mut self, preload_file_name: &str, assets_directory: &PathBuf) {
        
    }
}

impl ResourceSystem for AssetLoader {
    fn domain(&self) -> &str {
        &self.domain
    }
    fn categories(&self) -> &[crate::resource_loader::ResourceCategory] {
        &self.categories
    }
    fn try_load_file(&mut self, category: &str, namespace: &str, file_name: &str, file_extension: &str, contents: &[u8]) {
        log::warn!("loading {}:{}{} from {} ({file_name})", namespace, file_name, file_extension, category);
        match category {
            "models" => { self.try_load_model(namespace, file_name, contents); },
            "textures" => {},
            "blockstates" => { self.try_load_blockstate(namespace, file_name, contents); },
            _ => {},
        }
    }
}
