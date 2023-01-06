use std::collections::HashMap;

use crate::{block::Block, identifier::Identifier};


/** Represents the total registered blocks, items, [tile]entities, dimensions, and any other object
 * that needs to be referenced.
 *
 *
 */
pub struct Registry {
    // items: Vec<Item>,
    blocks: Register<Block>,
    // dimension: Vec<Dimension>,
}

impl Registry {
    /** Creates a new registery with set default capacities to prevent constant reallocation
     *
     */
    pub fn new() -> Self {
        let blocks = Register::<Block>::new(256);
        Self {
            blocks,
        }
    }

    pub fn get_block_register(&self) -> &Register<Block> {
        return &self.blocks;
    }

    pub fn get_block_register_mut(&mut self) -> &mut Register<Block> {
        return &mut self.blocks;
    }
}

/** Represents types that can be registered for indexing by indentifiers
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
        let id_map = HashMap::with_capacity(default_capacity);
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
        self.current_id +=1;
        self.collection.push(registerable);
        self.id_map.insert(identifier, index);
    }

    /** Clears the register, but keeps allocated memory
     * Might be used to implement run-time reloading of packs from the server
     *
     */
    pub fn reset(&mut self) {
        self.current_id = 0;
        self.collection.clear();
        self.id_map.clear();
    }

    pub fn get_element_from_identifier(&self, ident: impl Into<Identifier>) -> Option<&T> {
        let identifier: Identifier = ident.into();
        let default_index = 0;
        let index: usize = self.id_map.get(&identifier.to_string()).unwrap_or(&default_index) + 0;
        self.collection.get(index)
    }

    pub fn get_element_from_index(&self, index: usize) -> Option<&T> {
        self.collection.get(index)
    }

    pub fn get_index_from_identifier(&self, ident: impl Into<Identifier>) -> usize {
        let identifier = ident.into();
        let default_index = 0;
        self.id_map.get(&identifier.to_string()).unwrap_or(&default_index) + 0
    }
}
