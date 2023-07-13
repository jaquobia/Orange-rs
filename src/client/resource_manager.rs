use std::collections::HashMap;
use std::hash::Hash;
use crate::client::textures::TextureObject;

pub enum ResourceSource {
    Directory,
    Zip,
}

pub enum ResourceManagerReloadError {
}

pub type ResourceManagerReloadResult = Result<(), ResourceManagerReloadError>;


pub trait ResourceSystem {

}

pub struct ResourceManager<IdentifierType> {
    sources: Vec<ResourceSource>,
    systems: Vec<Box<dyn ResourceSystem>>,
    textures: HashMap<IdentifierType, TextureObject>,
}

impl<IdentifierType: Hash + PartialEq + Eq> ResourceManager<IdentifierType> {
    pub fn new() -> Self {
        Self {
            sources: vec![],
            systems: vec![],
            textures: HashMap::new(),
        }
    }
    ///
    pub fn reload() -> ResourceManagerReloadResult {
        unimplemented!()
    }
    /// Get a texture indexed from an IdentifierType
    pub fn get_texture(&self, id: IdentifierType) -> TextureObject {
        *self.textures.get(&id).unwrap()
    }
}