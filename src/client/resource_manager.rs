use std::collections::HashMap;
use std::hash::Hash;
use crate::client::textures::TextureObject;

pub struct ResourceManager<IdentifierType> {
    textures: HashMap<IdentifierType, TextureObject>,
}

impl<IdentifierType: Hash + PartialEq + Eq> ResourceManager<IdentifierType> {
    /// Get a texture indexed from an IdentifierType
    pub fn get_texture(&self, id: IdentifierType) -> TextureObject {
        *self.textures.get(&id).unwrap()
    }
}