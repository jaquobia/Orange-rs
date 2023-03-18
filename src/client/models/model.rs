use std::collections::HashMap;
use ultraviolet::{Vec2, Vec3};

pub struct VoxelModel {
    textures: HashMap<String, String>,
    elements: Vec<VoxelElement>,
    rotation: Option<VoxelRotation>,
}

pub struct VoxelElement {
    from: Vec3,
    to: Vec3,
    faces: [VoxelFace; 6]
}

pub struct VoxelFace {
    uv: [Vec2; 2],
    texture: String,
}

pub struct VoxelRotation {
    angle: f32,
    axis: u8,
    origin: Vec3,
}