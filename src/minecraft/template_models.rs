use lazy_static::lazy_static;
use crate::client::models::model::{VoxelElement, VoxelFace, VoxelModel, VoxelRotation};
use crate::direction::Direction;

lazy_static! {
    static ref TEMPLATE_MODELS: Vec<VoxelModel> = {
        let cube_template = VoxelModel::new().with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0])
        .with_face(VoxelFace::new("#down").with_cullface(Direction::Down), Direction::Down)
        .with_face(VoxelFace::new("#up").with_cullface(Direction::Up), Direction::Up)
        .with_face(VoxelFace::new("#north").with_cullface(Direction::North), Direction::North)
        .with_face(VoxelFace::new("#south").with_cullface(Direction::South), Direction::South)
        .with_face(VoxelFace::new("#west").with_cullface(Direction::West), Direction::West)
        .with_face(VoxelFace::new("#east").with_cullface(Direction::East), Direction::East)
        );

        let cube_all_template = VoxelModel::from_template(&cube_template)
        .with_texture("particle", "#all")
        .with_texture("down", "#all")
        .with_texture("up", "#all")
        .with_texture("north", "#all")
        .with_texture("south", "#all")
        .with_texture("west", "#all")
        .with_texture("east", "#all");

        let cube_column_template = VoxelModel::from_template(&cube_template)
        .with_texture("particle", "#up")
        .with_texture("down", "#up")
        .with_texture("north", "#side")
        .with_texture("south", "#side")
        .with_texture("west", "#side")
        .with_texture("east", "#side");

        let cross_template = VoxelModel::new()
        .with_ambient_occlusion(false)
        .with_texture("particle", "#cross")
        .with_element(VoxelElement::new([0.8, 0.0, 8.0], [15.2, 16.0, 8.0])
            .with_rotation(VoxelRotation::new(45.0, 1, [8.0, 8.0, 8.0], true))
            .with_shade(false)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
        ).with_element(VoxelElement::new([8.0, 0.0, 0.8], [8.0, 16.0, 15.2])
            .with_rotation(VoxelRotation::new(45.0, 1, [8.0, 8.0, 8.0], true))
            .with_shade(false)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
        );

        let crop_template = VoxelModel::new()
        .with_ambient_occlusion(false)
        .with_texture("particle", "#crop")
        .with_element(VoxelElement::new([4.0, -1.0, 0.0], [4.0, 15.0, 16.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::East)
        ).with_element(VoxelElement::new([12.0, -1.0, 0.0], [ 12.0, 15.0, 16.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::West)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
        ).with_element(VoxelElement::new([0.0, -1.0, 4.0], [ 16.0, 15.0, 4.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::South)
        ).with_element(VoxelElement::new([0.0, -1.0, 12.0], [16.0, 15.0, 12.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
        );
        let slab_template = VoxelModel::new().with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 8.0, 16.0])
        .with_face(VoxelFace::new("#down").with_cullface(Direction::Down), Direction::Down)
        .with_face(VoxelFace::new("#up").with_cullface(Direction::Up), Direction::Up)
        .with_face(VoxelFace::new("#north").with_cullface(Direction::North).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::North)
        .with_face(VoxelFace::new("#south").with_cullface(Direction::South).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::South)
        .with_face(VoxelFace::new("#west").with_cullface(Direction::West).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::West)
        .with_face(VoxelFace::new("#east").with_cullface(Direction::East).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::East)
        );
        let slab_all_template = VoxelModel::from_template(&slab_template)
        .with_texture("particle", "#all")
        .with_texture("down", "#all")
        .with_texture("up", "#all")
        .with_texture("north", "#all")
        .with_texture("south", "#all")
        .with_texture("west", "#all")
        .with_texture("east", "#all");
        let slab_column_template = VoxelModel::from_template(&slab_template)
        .with_texture("particle", "#up")
        .with_texture("down", "#up")
        .with_texture("north", "#side")
        .with_texture("south", "#side")
        .with_texture("west", "#side")
        .with_texture("east", "#side");
        vec![cube_template, cube_all_template, cross_template, crop_template, cube_column_template, slab_template, slab_all_template, slab_column_template]
    };
}

pub fn cube() -> &'static VoxelModel {
    &TEMPLATE_MODELS[0]
}

pub fn cube_all() -> &'static VoxelModel {
    &TEMPLATE_MODELS[1]
}

pub fn cross() -> &'static VoxelModel {
    &TEMPLATE_MODELS[2]
}

pub fn crop() -> &'static VoxelModel {
    &TEMPLATE_MODELS[3]
}

pub fn column() -> &'static VoxelModel {
    &TEMPLATE_MODELS[4]
}

pub fn slab() -> &'static VoxelModel {
    &TEMPLATE_MODELS[5]
}

pub fn slab_all() -> &'static VoxelModel {
    &TEMPLATE_MODELS[6]
}

pub fn slab_column() -> &'static VoxelModel {
    &TEMPLATE_MODELS[7]
}