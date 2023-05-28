use lazy_static::lazy_static;
use crate::block::properties::Property;
use crate::direction::Direction;



lazy_static! {
    pub static ref PROPERTY_NORTH: Property = Property::new_bool("minecraft:north");
    pub static ref PROPERTY_SOUTH: Property = Property::new_bool("minecraft:south");
    pub static ref PROPERTY_EAST: Property = Property::new_bool("minecraft:east");
    pub static ref PROPERTY_WEST: Property = Property::new_bool("minecraft:west");
    pub static ref PROPERTY_UP: Property = Property::new_bool("minecraft:up");
    pub static ref PROPERTY_DOWN: Property = Property::new_bool("minecraft:down");

    pub static ref PROPERTY_SNOWY: Property = Property::new_bool("minecraft:snowy");

    pub static ref PROPERTY_FACING: Property = Property::new_enum::<Direction, _>("minecraft:facing");
}