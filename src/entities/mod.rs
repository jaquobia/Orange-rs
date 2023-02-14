use ultraviolet::Vec3;

/// Entities are collections of components, able to function in an ECS
///

pub struct EntityTransform {
    pub position: Vec3,
    pub rotation: Vec3,
}

pub struct EntityMotion {
    pub velocity: Vec3,
}
