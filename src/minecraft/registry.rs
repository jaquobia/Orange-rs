use crate::block::BlockState;
use crate::block::properties::PropertyDefinition;
use crate::client::models::model::BakedModel;
use crate::{block::Block, minecraft::identifier::Identifier, game_version::GameVersion};
use crate::client::textures::TextureObject;

use rustc_hash::FxHashMap as HashMap;

/** Represents the total registered block, items, [tile]entities, dimensions, and any other object
 * that needs to be referenced.
 */

pub struct Registry {
    // items: Vec<Item>,
    blocks: Register<Block>,
    textures: HashMap<Identifier, TextureObject>,
    properties: Register<PropertyDefinition>,
    blockstates: Register<BlockState>,
    models: HashMap<Identifier, BakedModel>,
    // dimension: Vec<Dimension>,
}

impl Registry {
    /**
     * Creates a new registry
     * Automatically allocated space for 256 elements
     */
    pub fn new() -> Self {
        let blocks = Register::<Block>::new(256);
        let textures = HashMap::default();
        let properties = Register::<PropertyDefinition>::new(256);
        let blockstates = Register::<BlockState>::new(256);
        let models = HashMap::default();
        Self { blocks, textures, properties, blockstates, models }
    }

    pub fn load_from(version: GameVersion) -> Self {
        let mut registry = Self::new();
        
        version.load_registry(&mut registry);

        registry
    }

    pub fn load_custom<F: FnOnce(&mut Registry)>(funct: F) -> Self {
        let mut registry = Self::new();
        funct(&mut registry);
        registry
    }

    pub fn get_block_register(&self) -> &Register<Block> {
        return &self.blocks;
    }

    pub fn get_texture_register(&self) -> &HashMap<Identifier, TextureObject> {
        return &self.textures;
    }

    pub fn get_block_register_mut(&mut self) -> &mut Register<Block> {
        return &mut self.blocks;
    }

    pub fn get_texture_register_mut(&mut self) -> &mut HashMap<Identifier, TextureObject> {
        return &mut self.textures;
    }

    pub fn get_property_register(&self) -> &Register<PropertyDefinition> {
        &self.properties
    }

    pub fn get_property_register_mut(&mut self) -> &mut Register<PropertyDefinition> {
        &mut self.properties
    }

    pub fn get_blockstate_register(&self) -> &Register<BlockState> {
        &self.blockstates
    }

    pub fn get_blockstate_register_mut(&mut self) -> &mut Register<BlockState> {
        &mut self.blockstates
    }

    pub fn get_model_register(&self) -> &HashMap<Identifier, BakedModel> {
        &self.models
    }

    pub fn get_model_register_mut(&mut self) -> &mut HashMap<Identifier, BakedModel> {
        &mut self.models
    }

    pub fn reset(&mut self) {
        self.blocks.clear();
    }
}

/** Represents types that can be registered for indexing by identifiers
 *
 */
pub trait Registerable {
    fn get_identifier(&self) -> &Identifier;
}

/** A collection of a single registerable type
 *  Contains data for mapping indentifiers to the index of the element, and a counter for the next
 *  index,
 */
pub struct Register<T: Registerable> {
    collection: Vec<T>,
    id_map: HashMap<String, usize>,
    current_id: usize,
}

impl<T: Registerable> Register<T> {
    fn new(default_capacity: usize) -> Self {
        let collection = Vec::with_capacity(default_capacity);
        let id_map = HashMap::default();
        let current_id = 0;

        Self {
            collection,
            id_map,
            current_id,
        }
    }

    /** Puts the registerable into the register
     *
     */
    pub fn insert(&mut self, registerable: T) {
        let index = self.current_id;
        let identifier = registerable.get_identifier().to_string();
        self.current_id += 1;
        self.collection.push(registerable);
        self.id_map.insert(identifier, index);
    }

    /** Clears the register, but keeps allocated memory
     * Might be used to implement run-time reloading of packs from the server
     *
     */
    pub fn clear(&mut self) {
        self.current_id = 0;
        self.collection.clear();
        self.id_map.clear();
    }

    pub fn get_element_from_identifier(&self, ident: impl Into<Identifier>) -> Option<&T> {
        self.collection.get(self.get_index_from_identifier(ident))
    }

    pub fn get_element_from_index(&self, index: usize) -> Option<&T> {
        self.collection.get(index)
    }

    pub fn get_index_from_identifier(&self, ident: impl Into<Identifier>) -> usize {
        *self.id_map
            .get(&ident.into().to_string())
            .unwrap_or(&0)
    }

    pub fn get_elements(&self) -> &Vec<T> {
        &self.collection
    }
}
