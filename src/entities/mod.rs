use ultraviolet::Vec3;

/// Entities are collections of components, able to function in an ECS
///
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityTransform {
    pub position: Vec3,
    pub rotation: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityMotion {
    pub velocity: Vec3,
}

// Represents the target for camera transform
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityCamera {
}

// Represents the input target of the control system
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityController {
    pub on_ground: bool,
    pub stance: f64,
}
