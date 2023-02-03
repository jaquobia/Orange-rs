use crate::identifier::Identifier;

use self::dimension::Dimension;

pub mod chunk;
pub mod chunk_map;
pub mod dimension;
pub mod terrain_generator;


pub trait World {
    fn get_dimension_from_identifier(identifier: &Identifier);
    fn get_dimension_from_id(id: i32);
}

pub struct DedicatedServer {
    dimensions: Vec<Dimension>,
}

impl DedicatedServer {

    pub fn new() -> Self {
        let dimensions = vec![];
        Self {
            dimensions,
        }
    }
}

impl World for DedicatedServer {
    fn get_dimension_from_id(id: i32) {
        todo!();
    }
    fn get_dimension_from_identifier(identifier: &Identifier) {
        todo!();
    }
}
