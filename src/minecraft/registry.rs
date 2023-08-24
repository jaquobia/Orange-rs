use std::rc::Rc;

use crate::block::BlockState;
use crate::block::properties::PropertyDefinition;
use crate::models::model::BakedModel;
use crate::sprites::Sprite;
use crate::{block::Block, minecraft::identifier::Identifier};

use rustc_hash::FxHashMap as HashMap;

/** Represents the total registered block, items, [tile]entities, dimensions, and any other object
 * that needs to be referenced.
 */

pub type SpriteRegister = HashMap<Identifier, Sprite>;
pub type BlockRegister = Register<Block>;
pub type PropertyRegister = Register<PropertyDefinition>;
pub type BlockStateRegister = Register<BlockState>;
pub type ModelRegister = HashMap<Identifier, BakedModel>;

pub struct Registry {
    // items: Vec<Item>,
    blocks: BlockRegister,
    sprites: SpriteRegister,
    properties: PropertyRegister,
    blockstates: BlockStateRegister,
    models: ModelRegister,
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
        Self { blocks, sprites: textures, properties, blockstates, models }
    }

    pub fn get_block_register(&self) -> &BlockRegister {
        return &self.blocks;
    }

    pub fn get_sprite_register(&self) -> &SpriteRegister {
        return &self.sprites;
    }

    pub fn get_block_register_mut(&mut self) -> &mut BlockRegister {
        return &mut self.blocks;
    }

    pub fn get_sprite_register_mut(&mut self) -> &mut SpriteRegister {
        return &mut self.sprites;
    }

    pub fn get_property_register(&self) -> &PropertyRegister {
        &self.properties
    }

    pub fn get_property_register_mut(&mut self) -> &mut PropertyRegister {
        &mut self.properties
    }

    pub fn get_blockstate_register(&self) -> &BlockStateRegister {
        &self.blockstates
    }

    pub fn get_blockstate_register_mut(&mut self) -> &mut BlockStateRegister {
        &mut self.blockstates
    }

    pub fn get_model_register(&self) -> &ModelRegister {
        &self.models
    }

    pub fn get_model_register_mut(&mut self) -> &mut ModelRegister {
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
    collection: Vec<Rc<T>>,
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
    pub fn insert(&mut self, registerable: T) -> usize {
        let index = self.current_id;
        let identifier = registerable.get_identifier().to_string();
        self.current_id += 1;
        self.collection.push(Rc::new(registerable));
        self.id_map.insert(identifier, index);
        index
    }

    /** Puts the registerable into the register
     *
     */
    pub fn insert_pointer(&mut self, registerable: Rc<T>) -> usize {
        let index = self.current_id;
        let identifier = registerable.get_identifier().to_string();
        self.current_id += 1;
        self.collection.push(registerable.clone());
        self.id_map.insert(identifier, index);
        index
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

    pub fn get_element_from_identifier(&self, ident: &Identifier) -> Option<Rc<T>> {
        self.collection.get(self.get_index_from_identifier(ident)).cloned()
    }

    pub fn get_element_from_index(&self, index: usize) -> Option<Rc<T>> {
        self.collection.get(index).cloned()
    }

    pub fn get_index_from_identifier(&self, ident: &Identifier) -> usize {
        *self.id_map
            .get(&ident.to_string())
            .unwrap_or(&0)
    }

    pub fn get_elements(&self) -> &Vec<Rc<T>> {
        &self.collection
    }

    pub fn get_next_index(&self) -> usize {
        self.current_id
    }
}
