use lazy_static::lazy_static;
use ultraviolet::Vec3;
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
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
        ).with_element(VoxelElement::new([8.0, 0.0, 0.8], [8.0, 16.0, 15.2])
            .with_rotation(VoxelRotation::new(45.0, 1, [8.0, 8.0, 8.0], true))
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#cross").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
            .with_shade(false)
        );

        let crop_template = VoxelModel::new()
        .with_ambient_occlusion(false)
        .with_texture("particle", "#crop")
        .with_element(VoxelElement::new([4.0, -1.0, 0.0], [4.0, 15.0, 16.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::North)
        ).with_element(VoxelElement::new([12.0, -1.0, 0.0], [ 12.0, 15.0, 16.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::South)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
        ).with_element(VoxelElement::new([0.0, -1.0, 4.0], [ 16.0, 15.0, 4.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::West)
        ).with_element(VoxelElement::new([0.0, -1.0, 12.0], [16.0, 15.0, 12.0])
            .with_shade(false)
            .with_face(VoxelFace::new("#crop").with_uv([16.0, 0.0], [0.0, 16.0]), Direction::East)
            .with_face(VoxelFace::new("#crop").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
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

        let torch_template = VoxelModel::new()
        .with_element(VoxelElement::new([7.0, 0.0, 7.0], [9.0, 10.0, 9.0])
            .with_face(VoxelFace::new("#torch").with_uv([7.0, 6.0], [9.0, 8.0]), Direction::Up)
            .with_face(VoxelFace::new("#torch").with_uv([7.0, 14.0], [9.0, 16.0]), Direction::Down)
            .with_shade(false)
        ).with_element(VoxelElement::new([7.0, 0.0, 0.0], [9.0, 16.0, 16.0])
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
            .with_shade(false)
        ).with_element(VoxelElement::new([0.0, 0.0, 7.0], [16.0, 16.0, 9.0])
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
            .with_shade(false)
        ).with_ambient_occlusion(false);
        let wall_torch_template = VoxelModel::new()
        .with_element(VoxelElement::new([-1.0, 3.5, 7.0], [1.0, 13.5, 9.0])
            .with_face(VoxelFace::new("#torch").with_uv([7.0, 6.0], [9.0, 8.0]), Direction::Up)
            .with_face(VoxelFace::new("#torch").with_uv([7.0, 13.0], [9.0, 15.0]), Direction::Down)
            .with_rotation(VoxelRotation::new(-22.5, 2, Vec3::new(0.0, 3.5, 8.0), false))
            .with_shade(false)
        ).with_element(VoxelElement::new([-1.0, 3.5, 0.0], [1.0, 19.5, 16.0])
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::South)
            .with_rotation(VoxelRotation::new(-22.5, 2, Vec3::new(0.0, 3.5, 8.0), false))
            .with_shade(false)
        ).with_element(VoxelElement::new([-8.0, 3.5, 7.0], [8.0, 19.5, 9.0])
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::East)
            .with_face(VoxelFace::new("#torch").with_uv([0.0, 0.0], [16.0, 16.0]), Direction::West)
            .with_rotation(VoxelRotation::new(-22.5, 2, Vec3::new(0.0, 3.5, 8.0), false))
            .with_shade(false)
        ).with_ambient_occlusion(false);
        let missing_template = VoxelModel::from_template(&cube_all_template).with_texture("all", "missing");
        let orientable_template = VoxelModel::from_template(&cube_template)
        .with_texture("north", "#front")
        .with_texture("south", "#side")
        .with_texture("east", "#side")
        .with_texture("west", "#side");
        let column_top_bottom_template = VoxelModel::from_template(&cube_template)
        .with_texture("particle", "#up")
        .with_texture("north", "#side")
        .with_texture("south", "#side")
        .with_texture("west", "#side")
        .with_texture("east", "#side");
        let stair_template = VoxelModel::new()
        .with_texture("particle", "#all")
        .with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 8.0, 16.0])
            .with_face(VoxelFace::new("#down").with_cullface(Direction::Down), Direction::Down)
            .with_face(VoxelFace::new("#up").with_cullface(Direction::Up), Direction::Up)
            .with_face(VoxelFace::new("#north").with_cullface(Direction::North).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::North)
            .with_face(VoxelFace::new("#south").with_cullface(Direction::South).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::South)
            .with_face(VoxelFace::new("#west").with_cullface(Direction::West).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::West)
            .with_face(VoxelFace::new("#east").with_cullface(Direction::East).with_uv([0.0, 0.0], [16.0, 8.0]), Direction::East)
        )
        .with_element(VoxelElement::new([8.0, 8.0, 0.0], [16.0, 16.0, 16.0])
            .with_face(VoxelFace::new("#up").with_cullface(Direction::Up).with_uv([0.0, 8.0], [16.0, 16.0]), Direction::Up)
            .with_face(VoxelFace::new("#north").with_uv([0.0, 8.0], [16.0, 16.0]), Direction::North)
            .with_face(VoxelFace::new("#south").with_cullface(Direction::South).with_uv([0.0, 8.0], [16.0, 16.0]), Direction::South)
            .with_face(VoxelFace::new("#west").with_cullface(Direction::West).with_uv([8.0, 8.0], [16.0, 16.0]), Direction::West)
            .with_face(VoxelFace::new("#east").with_cullface(Direction::East).with_uv([0.0, 8.0], [8.0, 16.0]), Direction::East)
        );
        let stair_all_template = VoxelModel::from_template(&stair_template)
        .with_texture("particle", "#all")
        .with_texture("north", "#all")
        .with_texture("south", "#all")
        .with_texture("east", "#all")
        .with_texture("west", "#all")
        .with_texture("up", "#all")
        .with_texture("down", "#all");
        vec![cube_template, cube_all_template, cross_template, crop_template, cube_column_template, slab_template, slab_all_template, slab_column_template, torch_template, missing_template, wall_torch_template, orientable_template, column_top_bottom_template, stair_template, stair_all_template]
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

pub fn torch() -> &'static VoxelModel {
    &TEMPLATE_MODELS[8]
}

pub fn missing() -> &'static VoxelModel {
    &TEMPLATE_MODELS[9]
}

pub fn wall_torch() -> &'static VoxelModel {
    &TEMPLATE_MODELS[10]
}

pub fn orientable() -> &'static VoxelModel {
    &TEMPLATE_MODELS[11]
}
pub fn column_top_bottom() -> &'static VoxelModel {
    &TEMPLATE_MODELS[12]
}
pub fn stair() -> &'static VoxelModel {
    &TEMPLATE_MODELS[13]
}
pub fn stair_all() -> &'static VoxelModel {
    &TEMPLATE_MODELS[14]
}