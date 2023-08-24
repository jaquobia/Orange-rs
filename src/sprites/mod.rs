use ultraviolet::Vec2;

use crate::minecraft::identifier::Identifier;

pub struct Sprite {
    pub parent_texture: Identifier,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}
