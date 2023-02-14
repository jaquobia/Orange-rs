use crate::identifier::Identifier;

use self::dimension::Dimension;

pub mod chunk;
pub mod chunk_map;
pub mod dimension;
pub mod terrain_generator;

pub struct World {
    dimensions: Vec<Dimension>,
}

impl World {

    pub fn new() -> Self {
        let dimensions = vec![];
        Self {
            dimensions,
        }
    }

}

pub trait WorldTrait {
    fn get_dimension_from_identifier(&self, identifier: &Identifier) -> Option<&Dimension>;
    fn get_dimension_from_id(&self, id: i32) -> Option<&Dimension>;
}

impl WorldTrait for World {

    fn get_dimension_from_id(&self, id: i32) -> Option<&Dimension> {
        for dimension in &self.dimensions {
            if dimension.id == id {
                return Some(&dimension)
            }
        }
        return None;
    }

    fn get_dimension_from_identifier(&self, identifier: &Identifier) -> Option<&Dimension> {
        for dimension in &self.dimensions {
            if dimension.identifier == *identifier {
                return Some(&dimension);
            }
        }
        return None;
    }
}
