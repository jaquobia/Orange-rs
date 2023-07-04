use std::fs::{self, DirEntry};
use std::path::PathBuf;

use rustc_hash::FxHashMap as HashMap;
use ultraviolet::{Vec2, Vec3};
use crate::block::Block;
use crate::block::block_factory::BlockFactory;
use crate::block::properties::PropertyDefinition;
use crate::client::models::model::{BakedModel, VoxelElement, VoxelFace, VoxelModel, VoxelRotation};
use crate::client::textures::TextureObject;
use crate::client::textures::TextureObject::AtlasTexture;
use crate::direction::Direction;
use crate::minecraft::filetypes::{MCAtlasTextureFile, UniformAtlasTextureType, MCModelFile, MCBlockstateType};
use crate::minecraft::identifier::Identifier;
use crate::minecraft::registry::Registry;
use crate::minecraft::template_models::{cube_all, door_bottom_left, door_bottom_right, door_top_left, door_top_right, missing, orientable, pressure_plate_down, pressure_plate_up, torch, wall_torch};

pub enum GameVersion {
    B173,
    Orange,
}

impl GameVersion {
    pub fn load_registry(&self, registry: &mut Registry) {
        match self {
            Self::B173 => load_b173(registry),
            _ => {},
        }
    }
}

fn get_uv_from_atlas_index(texture_index: usize) -> [Vec2; 2] {
    let (u, v) = ((texture_index % 16) as f32 * 16., (texture_index / 16) as f32 * 16.,);
    let (u, v) = ([u, v], [u + 16., v + 16.]);
    const INV_ATLAS_SIZE: f32 = 1.0 / 256.;
    [Vec2::new((u[0] * INV_ATLAS_SIZE) as f32, (u[1] * INV_ATLAS_SIZE) as f32), Vec2::new((v[0] * INV_ATLAS_SIZE) as f32, (v[1] * INV_ATLAS_SIZE) as f32)]
}

fn make_atlas_tex(texture_index: usize) -> TextureObject {
    AtlasTexture { internal_uv: get_uv_from_atlas_index(texture_index) }
}

fn slab_cull(dir: Direction) -> bool {
    dir == Direction::Down
}

fn non_full_cull(_: Direction) -> bool {
    false
}


/** Apply a function to all files in dir and subdirs   
Will crash if depth is greater than number of allowed open files per program 
*/
fn iter_files_recursive<F: FnMut(&DirEntry)>(path: PathBuf, file_funct: &mut F) {
    if !path.is_dir() {
        log::error!("Not a dir: {}", path.display());
        return;
    }

    for f in fs::read_dir(path).unwrap() {
        let entry = f.unwrap();
        let entry_path = entry.path();
        if entry_path.is_dir() {
            iter_files_recursive(entry_path, file_funct);
        } else {
            file_funct(&entry);
        }
    }
}

fn register_properties(registry: &mut Registry) {

    let properties = registry.get_property_register_mut();
    let property_list: &[(&str, &[&str])] = &[
        ("minecraft:boolean", &["false", "true"]),
        ("minecraft:block_half", &["bottom", "top"]),
        ("minecraft:redstone_side", &["side", "up"]),
        ("minecraft:count_1", &["0", "1"]),
        ("minecraft:count_2", &["0", "1", "2"]),
        ("minecraft:count_3", &["0", "1", "2", "3"]),
        ("minecraft:count_4", &["0", "1", "2", "3", "4"]),
        ("minecraft:count_5", &["0", "1", "2", "3", "4", "5"]),
        ("minecraft:count_6", &["0", "1", "2", "3", "4", "5", "6"]),
        ("minecraft:count_7", &["0", "1", "2", "3", "4", "5", "6", "7"]),
        ("minecraft:count_15", &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"]),
        ("minecraft:facing", &["north", "south", "east", "west", "up", "down"]),
        ("minecraft:facing_horizontal", &["north", "south", "east", "west"]),
        ("minecraft:bed_part", &["foot", "head"]),
        ("minecraft:tall_grass_type", &["grass", "fern", "dead_bush"]),
        ("minecraft:tree_type", &["oak", "spruce", "birch"]),
        ("minecraft:color", &["white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black"]),
    ];
    for property_def in property_list {
        properties.insert(PropertyDefinition::new(property_def.0.into(), property_def.1));
    }
}

fn register_blocks(registry: &mut Registry) {
let block_register_list = vec![
            BlockFactory::new("air")
                .hardness(0.0)
                .resistance(0.0)
                .transparent(true)
                .full_block(false)
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("stone")
                .hardness(1.5)
                .resistance(10.0)
                .build(),
            BlockFactory::new("grass")
                .hardness(0.6)
                .properties(&vec![("snowy", "minecraft:boolean")])
                .build(),
            BlockFactory::new("dirt")
                .hardness(0.5)
                .build(),
            BlockFactory::new("cobblestone")
                .hardness(2.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("oak_planks") // planks
                .hardness(2.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("sapling")
                .hardness(0.0)
                .properties(&vec![("tree", "minecraft:tree_type"), ("growth", "minecraft:count_1")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("bedrock")
                .hardness(-1.0)
                .resistance(6000000.0)
                .build(),
            BlockFactory::new("flowing_water")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:water_0").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("still_water")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:water_0").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("flowing_lava")
                .hardness(0.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:lava_0").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("still_lava")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:lava_0").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("sand")
                .hardness(0.5)
                .build(),
            BlockFactory::new("gravel")
                .hardness(0.5)
                .build(),
            BlockFactory::new("ore_gold")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("ore_iron")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("ore_coal")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("log")
                .hardness(2.0)
                .properties(&vec![("tree", "minecraft:tree_type")])
                .build(),
            BlockFactory::new("leaves")
                .hardness(0.2)
                .properties(&vec![("tree", "minecraft:tree_type"), ("decay", "minecraft:count_1")])
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("sponge")
                .hardness(0.6)
                .build(),
            BlockFactory::new("glass")
                .hardness(0.3)
                .resistance(6000000.0)
                .transparent(true)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("ore_lapis")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("block_lapis")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("dispenser")
                .hardness(3.5)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .model(|meta| {
                    let rotation = match meta {
                        2 => 270.,
                        3 => 90.,
                        4 => 0.,
                        5 => 180.,
                        _ => return missing().clone().bake(),
                    };
                    orientable().clone()
                        .with_texture("up", "minecraft:furnace_top")
                        .with_texture("down", "minecraft:furnace_top")
                        .with_texture("front", "minecraft:dispenser_front")
                        .with_texture("side", "minecraft:furnace_side")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("sandstone")
                .hardness(0.8)
                .build(),
            BlockFactory::new("noteblock")
                .hardness(0.8)
                .build(),
            BlockFactory::new("bed")
                .hardness(0.2)
                .properties(&vec![("facing", "minecraft:facing_horizontal"), ("part", "minecraft:bed_part"), ("occupied", "minecraft:boolean")])
                .model(|meta| {
                    let rotation = match meta & 3 {
                        0 => 0.,
                        1 => 270.,
                        2 => 180.,
                        3 => 90.,
                        _ => 0.,
                    };
                    match meta & 8 == 0 {
                        true => {
                            // foot
                            VoxelModel::new()
                                .with_element(VoxelElement::new([0.0, 3.0, 0.0], [16.0, 10.0, 16.0])
                                    .with_face(VoxelFace::new("minecraft:bed_top_foot"), Direction::Up)
                                    .with_face(VoxelFace::new("minecraft:oak_plank"), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([0., 7.], [16., 13.]).with_cullface(Direction::North), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([16., 7.], [0., 13.]).with_cullface(Direction::South), Direction::South)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([0., 7.], [16., 13.]).with_cullface(Direction::East), Direction::East)
                                    // .with_face(VoxelFace::new("minecraft:bed_top_head").with_uv([16., 7.], [0., 13.]).with_cullface(Direction::West), Direction::West)
                                )
                                .with_element(VoxelElement::new([0.0, 0.0, 0.0], [3.0, 3.0, 3.0])
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([0., 13.], [3., 16.]).with_cullface(Direction::Down), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([0., 13.], [3., 16.]), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([3., 13.], [0., 16.]), Direction::South)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([13., 13.], [16., 16.]), Direction::East)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([3., 13.], [0., 16.]), Direction::West)
                                )
                                .with_element(VoxelElement::new([13.0, 0.0, 0.0], [16.0, 3.0, 3.0])
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([0., 13.], [3., 16.]).with_cullface(Direction::Down), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([3., 13.], [0., 16.]), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot").with_uv([3., 13.], [0., 16.]), Direction::South)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([0., 13.], [3., 16.]), Direction::East)
                                    .with_face(VoxelFace::new("minecraft:bed_side_foot_front").with_uv([13., 13.], [16., 16.]), Direction::West)
                                )
                                .with_ambient_occlusion(false)
                                .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                        },
                        false => {
                            // head
                            VoxelModel::new()
                                .with_element(VoxelElement::new([0.0, 3.0, 0.0], [16.0, 10.0, 16.0])
                                    .with_face(VoxelFace::new("minecraft:bed_top_head"), Direction::Up)
                                    .with_face(VoxelFace::new("minecraft:oak_plank"), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([0., 7.], [16., 13.]).with_cullface(Direction::North), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([16., 7.], [0., 13.]).with_cullface(Direction::South), Direction::South)
                                    // .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([0., 7.], [16., 13.]).with_cullface(Direction::East), Direction::East)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 7.], [16., 13.]).with_cullface(Direction::West), Direction::West)
                                )
                                .with_element(VoxelElement::new([0.0, 0.0, 13.0], [3.0, 3.0, 16.0])
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 13.], [3., 16.]).with_cullface(Direction::Down), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([13., 13.], [16., 16.]), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([16., 13.], [13., 16.]), Direction::South)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([3., 13.], [0., 16.]), Direction::East)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 13.], [3., 16.]), Direction::West)
                                )
                                .with_element(VoxelElement::new([13.0, 0.0, 13.0], [16.0, 3.0, 16.0])
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 13.], [3., 16.]).with_cullface(Direction::Down), Direction::Down)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([16., 13.], [13., 16.]), Direction::North)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head").with_uv([16., 13.], [13., 16.]), Direction::South)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 13.], [3., 16.]), Direction::East)
                                    .with_face(VoxelFace::new("minecraft:bed_side_head_front").with_uv([0., 13.], [3., 16.]), Direction::West)
                                )
                                .with_ambient_occlusion(false)
                                .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                        }
                    }
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("powered_rail")
                .hardness(0.7)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("detector_rail")
                .hardness(0.7)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("sticky_piston")
                .side_cull_fn(non_full_cull)
                .properties(&vec![("facing", "minecraft:facing")])
                .build(),
            BlockFactory::new("web")
                .hardness(4.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("tall_grass")
                .hardness(0.0)
                .properties(&vec![("type", "minecraft:tall_grass_type")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("dead_bush")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("piston")
                .properties(&vec![("facing", "minecraft:facing")])
                .model(|meta| {
                    log::warn!("piston meta: {meta}");
                    let is_powered = meta  & 8 > 0;
                    let rotation = match meta & 7 {
                        0 => { Some(VoxelRotation::new(90., 2, [8.0, 8.0, 8.0], false)) },
                        1 => { Some(VoxelRotation::new(270., 2, [8.0, 8.0, 8.0], false)) },
                        2 => { Some(VoxelRotation::new(270., 1, [8.0, 8.0, 8.0], false)) },
                        3 => { Some(VoxelRotation::new(90., 1, [8.0, 8.0, 8.0], false)) },
                        4 => { Some(VoxelRotation::new(0., 1, [8.0, 8.0, 8.0], false)) },
                        5 => { Some(VoxelRotation::new(180., 1, [8.0, 8.0, 8.0], false)) },
                        _ => return missing().clone().bake(),
                    };
                    let model = if is_powered {
                        VoxelModel::new().with_element(VoxelElement::new([4., 0., 0.], [16., 16., 16.])
                            .with_face(VoxelFace::new("minecraft:piston_base"), Direction::North)
                            .with_face(VoxelFace::new("minecraft:piston_bottom").with_cullface(Direction::South), Direction::South)
                            .with_face(VoxelFace::new("minecraft:piston_side").with_uv([0., 4.], [16., 16.]).with_rotation(270.).with_cullface(Direction::East), Direction::East)
                            .with_face(VoxelFace::new("minecraft:piston_side").with_uv([0., 4.], [16., 16.]).with_rotation(180.).with_cullface(Direction::West), Direction::West)
                            .with_face(VoxelFace::new("minecraft:piston_side").with_uv([0., 4.], [16., 16.]).with_rotation(180.).with_cullface(Direction::Up), Direction::Up)
                            .with_face(VoxelFace::new("minecraft:piston_side").with_uv([0., 4.], [16., 16.]).with_rotation(180.).with_cullface(Direction::Down), Direction::Down)
                        )
                    } else {
                        VoxelModel::new().with_element(VoxelElement::new([0., 0., 0.], [16., 16., 16.])
                            .with_face(VoxelFace::new("minecraft:piston_front").with_cullface(Direction::North), Direction::North)
                            .with_face(VoxelFace::new("minecraft:piston_bottom").with_cullface(Direction::South), Direction::South)
                            .with_face(VoxelFace::new("minecraft:piston_side").with_rotation(270.).with_cullface(Direction::East), Direction::East)
                            // .with_face(VoxelFace::new("minecraft:piston_side").with_rotation(270.).with_cullface(Direction::West), Direction::West)
                            // .with_face(VoxelFace::new("minecraft:piston_side").with_rotation(180.).with_cullface(Direction::Up), Direction::Up)
                            // .with_face(VoxelFace::new("minecraft:piston_side").with_rotation(180.).with_cullface(Direction::Down), Direction::Down)
                        )

                    }.bake_with_rotate(rotation);
                    log::warn!("Piston model end");
                    model
                    // 0 -> down
                    // 1 -> up
                    // 4 -> north
                    // 5 -> south
                    // 3 -> west
                    // 2 -> east
                    // TODO: COMPLEX MODEL
                    // cube_all().clone().with_texture("all", "minecraft:piston_front").bake()
                })
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("piston_extension")
                .properties(&vec![("facing", "minecraft:facing")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("wool")
                .hardness(0.8)
                .properties(&vec![("color", "minecraft:color")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("yellow_flower")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("red_flower")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("brown_mushroom")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("red_mushroom")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("block_gold")
                .hardness(3.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("block_iron")
                .hardness(5.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("double_stair") // double stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("single_stair") // single stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .side_cull_fn(slab_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("brick_block")
                .hardness(2.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("tnt")
                .hardness(0.0)
                .build(),
            BlockFactory::new("bookshelf")
                .hardness(1.5)
                .build(),
            BlockFactory::new("mossy_cobblestone")
                .hardness(2.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("obsidian")
                .hardness(10.0)
                .resistance(2000.0)
                .build(),
            BlockFactory::new("torch")
                .hardness(0.0)
                .model(|meta| {
                    match meta {
                        5 => VoxelModel::from_template(torch()).with_texture("torch", "minecraft:torch").bake(),
                        1 => VoxelModel::from_template(wall_torch()).with_texture("torch", "minecraft:torch").bake(),
                        2 => VoxelModel::from_template(wall_torch()).with_texture("torch", "minecraft:torch")
                            .bake_with_rotate(Some(VoxelRotation::new(180., 1, Vec3::new(8.0, 8.0, 8.0), false))),
                        3 => VoxelModel::from_template(wall_torch()).with_texture("torch", "minecraft:torch")
                            .bake_with_rotate(Some(VoxelRotation::new(270., 1, Vec3::new(8.0, 8.0, 8.0), false))),
                        4 => VoxelModel::from_template(wall_torch()).with_texture("torch", "minecraft:torch")
                            .bake_with_rotate(Some(VoxelRotation::new(90., 1, Vec3::new(8.0, 8.0, 8.0), false))),
                        _ => missing().clone().bake(),
                    }
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("fire")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                // TODO: ODD MODEL
                .build(),
            BlockFactory::new("mob_spawner")
                .hardness(5.0)
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("wooden_stairs")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("chest")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .hardness(2.5)
                .build(),
            BlockFactory::new("redstone_dust")
                .hardness(0.0)
                // TODO: COMPLEX MODEL
                .full_block(false)
                .build(),
            BlockFactory::new("ore_diamond")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("block_diamond")
                .hardness(5.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("workbench")
                .hardness(2.5)
                .build(),
            BlockFactory::new("crops")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("farmland")
                .hardness(0.6)
                .properties(&vec![("moisture", "minecraft:count_7")])
                .side_cull_fn(|dir| dir == Direction::Down)
                .build(),
            BlockFactory::new("furnace")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .hardness(3.5)
                .model(|meta| {
                    let rotation = match meta {
                        2 => 270.,
                        3 => 90.,
                        4 => 0.,
                        5 => 180.,
                        _ => return missing().clone().bake(),
                    };
                    orientable().clone()
                        .with_texture("up", "minecraft:furnace_top")
                        .with_texture("down", "minecraft:furnace_top")
                        .with_texture("front", "minecraft:furnace_front")
                        .with_texture("side", "minecraft:furnace_side")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("furnace_active")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .hardness(3.5)
                .model(|meta| {
                    let rotation = match meta {
                        2 => 270.,
                        3 => 90.,
                        4 => 0.,
                        5 => 180.,
                        _ => return missing().clone().bake(),
                    };
                    orientable().clone()
                        .with_texture("up", "minecraft:furnace_top")
                        .with_texture("down", "minecraft:furnace_top")
                        .with_texture("front", "minecraft:furnace_front_lit")
                        .with_texture("side", "minecraft:furnace_side")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("sign")
                .hardness(1.0)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("wooden_door")
                .hardness(3.0)
                .side_cull_fn(non_full_cull)
                .properties(&vec![("facing", "minecraft:facing_horizontal"), ("half", "minecraft:block_half"), ("powered", "minecraft:boolean")])
                .model(|meta| {
                    let is_bottom = meta & 8 == 0;
                    // is_open and is_right are the same thing in b1.7.3
                    let is_open = meta & 4 > 0;
                    let is_right = is_open;
                    let rotation_modifier = (meta & 4) >> 2;
                    let angle = (((meta & 3) + rotation_modifier) & 3) as f32 * -90.;
                    let voxel_rotation = Some(VoxelRotation::new(angle, 1, [8., 8., 8.], false));

                    if is_bottom {
                        if is_open {
                            door_bottom_right().clone()
                        } else {
                            door_bottom_left().clone()
                        }.with_texture("door_face", "minecraft:oak_door_bottom")
                    } else { // is top
                        if is_open {
                            door_top_right().clone()
                        } else {
                            door_top_left().clone()
                        }.with_texture("door_face", "minecraft:oak_door_top")
                    }.bake_with_rotate(voxel_rotation)
                })
                .full_block(false)
                .build(),
            BlockFactory::new("ladder")
                .hardness(0.4)
                .side_cull_fn(non_full_cull)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .full_block(false)
                .build(),
            BlockFactory::new("rail")
                .hardness(0.7)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("cobblestone_stair")
                .hardness(3.0)
                .build(),
            BlockFactory::new("wall_sign")
                .hardness(1.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("lever")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("stone_pressure_plate")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .model(|meta| {
                    if meta == 0 {
                        pressure_plate_up().clone()
                    } else {
                        pressure_plate_down().clone()
                    }.with_texture("all", "minecraft:stone").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("iron_door")
                .hardness(3.0)
                .side_cull_fn(non_full_cull)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .model(|meta| {
                    let is_bottom = meta & 8 == 0;
                    // is_open and is_right are the same thing in b1.7.3
                    let is_open = meta & 4 > 0;
                    let is_right = is_open;
                    let rotation_modifier = if is_open { 1 } else { 0 };
                    let angle = (((meta & 3) + rotation_modifier) & 3) as f32 * -90.;
                    let voxel_rotation = Some(VoxelRotation::new(angle, 1, [8., 8., 8.], false));

                    if is_bottom {
                        if is_open {
                            door_bottom_right().clone()
                        } else {
                            door_bottom_left().clone()
                        }.with_texture("door_face", "minecraft:iron_door_bottom")
                    } else { // is top
                        if is_open {
                            door_top_right().clone()
                        } else {
                            door_top_left().clone()
                        }.with_texture("door_face", "minecraft:iron_door_top")
                    }.bake_with_rotate(voxel_rotation)
                })
                .build(),
            BlockFactory::new("wooden_pressure_plate")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .model(|meta| {
                    if meta == 0 {
                        pressure_plate_up().clone()
                    } else {
                        pressure_plate_down().clone()
                    }.with_texture("all", "minecraft:oak_plank").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("ore_redstone")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("ore_redstone_glowing")
                .hardness(3.0)
                .resistance(5.0)
                .build(),
            BlockFactory::new("torch_redstone_off")
                .hardness(0.0)
                .model(|_| {
                    torch().clone().with_texture("torch", "minecraft:redstone_torch_off").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("torch_redstone_on")
                .hardness(0.0)
                .model(|_| {
                    torch().clone().with_texture("torch", "minecraft:redstone_torch_on").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("button")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("snow_layer")
                .hardness(0.1)
                .side_cull_fn(slab_cull)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("ice")
                .hardness(0.5)
                .transparent(true)
                // .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("snow")
                .hardness(0.2)
                .build(),
            BlockFactory::new("cactus")
                .hardness(0.4)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("clay_block")
                .hardness(0.6)
                .resistance(6000000.0)
                .build(),
            BlockFactory::new("reed")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("jukebox")
                .hardness(2.0)
                .resistance(10.0)
                .build(),
            BlockFactory::new("fence")
                .hardness(2.0)
                .resistance(5.0)
                .properties(&vec![("north", "minecraft:boolean"), ("south", "minecraft:boolean"), ("east", "minecraft:boolean"), ("west", "minecraft:boolean")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("pumpkin")
                .hardness(1.0)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .model(|meta| {
                    let rotation = match meta {
                        0 => 90.,
                        1 => 0.,
                        2 => 270.,
                        3 => 180.,
                        _ => 0.
                    };
                    cube_all().clone()
                        .with_texture("north", "minecraft:pumpkin_front")
                        .with_texture("up", "minecraft:pumpkin_top")
                        .with_texture("down", "minecraft:pumpkin_side")
                        .with_texture("south", "minecraft:pumpkin_side")
                        .with_texture("east", "minecraft:pumpkin_side")
                        .with_texture("west", "minecraft:pumpkin_side").bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8., 8., 8.], false)))
                })
                .build(),
            BlockFactory::new("netherrack")
                .hardness(0.4)
                .build(),
            BlockFactory::new("soulsand")
                .hardness(0.5)
                .build(),
            BlockFactory::new("glowstone_block")
                .hardness(0.3)
                .build(),
            BlockFactory::new("portal")
                .hardness(-1.0)
                .transparent(true)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("pumpkin_lantern")
                .hardness(1.0)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .model(|meta| {
                    let rotation = match meta {
                        0 => 90.,
                        1 => 0.,
                        2 => 270.,
                        3 => 180.,
                        _ => 0.
                    };
                    cube_all().clone()
                        .with_texture("north", "minecraft:lantern_front")
                        .with_texture("up", "minecraft:pumpkin_top")
                        .with_texture("down", "minecraft:pumpkin_side")
                        .with_texture("south", "minecraft:pumpkin_side")
                        .with_texture("east", "minecraft:pumpkin_side")
                        .with_texture("west", "minecraft:pumpkin_side").bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8., 8., 8.], false)))
                })
                .build(),
            BlockFactory::new("cake")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("repeater_off")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("repeater_on")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("locked_chest")
                .hardness(0.0)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .build(),
            BlockFactory::new("trapdoor")
                .hardness(-1.0)
                .properties(&vec![("facing", "minecraft:facing_horizontal"), ("half", "minecraft:block_half")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
        ];

        for block in block_register_list {
            let block_id = registry.get_block_register_mut().insert(block);
            let block = registry.get_block_register_mut().get_element_from_index(block_id).unwrap();
            for state in Block::map_states(block, registry) {
                registry.get_blockstate_register_mut().insert_pointer(state);
            }
        }
        log::warn!("There are {} blocks", registry.get_block_register().get_elements().len());
        log::warn!("There are {} blockstates", registry.get_blockstate_register().get_elements().len());

}

// TODO: Check for infinite recursion through already visited models
fn make_model(registry: &Registry, identifier: &Identifier, model_files: &HashMap<Identifier, MCModelFile>, voxel_models: &mut HashMap<Identifier, VoxelModel>) -> Option<VoxelModel> {
    let already_visited = false;
    if already_visited { return None; }

    if !model_files.contains_key(identifier) {
        log::warn!("No model file for {}", identifier);
    }
    model_files.get(identifier).and_then(|model_file| {
        let mut model = match model_file.get_parent() {
            Some(parent) => {
                // Apply models ontop of parent
                let parent_id: Identifier = parent.into();
                let parent_model = if voxel_models.contains_key(&parent_id) {
                    voxel_models.get(&parent_id)
                } else {
                    make_model(registry, &parent_id, model_files, voxel_models).and_then(|model| {
                        voxel_models.insert(parent_id.clone(), model);
                        voxel_models.get(&parent_id)
                    })
                };
                parent_model.and_then(|parent| {
                    let mut model = VoxelModel::from_template(parent);
                    if model_file.elements().len() > 0 {
                        model.clear_elements(); // override parent elements
                    }
                    
                    Some(model)
                })
            }, // end has a parent
            None => {
                // Create model file as-is
                let mut model = VoxelModel::new();
                model.with_ambient_occlusion_nc(model_file.get_ambient_occlusion());
                Some(model)
            } // end has no parent
        }; // end match parent

        if let Some(model) = &mut model {
            for (texture_var, texture_id) in model_file.textures() {
                model.with_texture_nc(texture_var, texture_id);
            }
            for element in model_file.elements() {
                let voxel_element = element.to_voxel_element();
                model.with_element_nc(voxel_element);
            }
        }

        model
    })
}

fn load_b173(registry: &mut Registry) {

    register_properties(registry);
    register_blocks(registry);

    let textures = registry.get_texture_register_mut();

    let mut model_files = HashMap::default();
    let mut voxel_models = HashMap::default();
    let mut blockstate_files = HashMap::default();

    let namespace = "minecraft";
    let context = "models";
    let assets_dir = ".";

    let read_dir = format!("{}/assets/{}/{}/", assets_dir, namespace, context);
    iter_files_recursive(read_dir.clone().into(), &mut |entry| {
        if entry.file_name().to_str().unwrap().to_owned().ends_with(".json") {
            match serde_json::from_str::<MCModelFile>(fs::read_to_string(entry.path()).unwrap().as_str()) {
                Ok(model_file) => {
                    let resource_extension = entry.path().extension().map(|ext| format!(".{}", ext.to_string_lossy().to_string())).unwrap_or("".to_string());
                    let resource_path = entry.path().to_string_lossy().replace(&read_dir, "").replace(&resource_extension, "");
                    let resource_id = Identifier::new(namespace.to_string(), resource_path.clone());
                    // log::warn!("Adding block model {} with id {}", resource_path, resource_id);
                    model_files.insert(resource_id, model_file);
                },
                Err(e) => { log::error!("Error processing {}: {}", entry.file_name().to_string_lossy(), e) }
            };
        } else {
            log::error!("Unknown File: {}", entry.path().display());
        }
    });

    let context = "blockstates";
    let read_dir = format!("{}/assets/{}/{}/", assets_dir, namespace, context);
    iter_files_recursive(read_dir.clone().into(), &mut |entry| {
        if entry.file_name().to_string_lossy().ends_with(".json") {
            match serde_json::from_str::<MCBlockstateType>(fs::read_to_string(entry.path()).unwrap().as_str()) {
                Ok(blockstate_file) => {
                    let resource_extension = entry.path().extension().map(|ext| format!(".{}", ext.to_string_lossy().to_string())).unwrap_or("".to_string());
                    let resource_path = entry.path().to_string_lossy().replace(&read_dir, "").replace(&resource_extension, "");
                    let resource_id = Identifier::new(namespace.to_string(), resource_path);
                    blockstate_files.insert(resource_id, blockstate_file);
                },
                Err(e) => { log::error!("Error processing {}: {}", entry.file_name().to_string_lossy(), e) }
            };
        }
    });

    let atlas_texture_json_str = fs::read_to_string("assets/minecraft/textures/block/terrain.mcatlas")
        .expect("Should have been able to read the file");
    let atlas_textures: MCAtlasTextureFile = serde_json::from_str(atlas_texture_json_str.as_str()).unwrap();

    for UniformAtlasTextureType { identifier, cell } in atlas_textures.atlas.get_uniform_textures() {
        let tex = make_atlas_tex(cell as usize);
        textures.insert(Identifier::from_str(identifier.as_str()), tex);
    }

    for model_file_id in model_files.keys() {
        let voxel_model = make_model(registry, &model_file_id, &model_files, &mut voxel_models);
        if let Some(voxel_model) = voxel_model { voxel_models.insert(model_file_id.clone(), voxel_model); } else { log::warn!("Couln't make voxel model for {}", model_file_id); }
    }
    

    // Blocks & Items
    

    let missing_model_file = voxel_models.get(&Identifier::from_str("minecraft:block/missing")).expect("Did not have a missing model");

    let mut mapped_models: Vec<(Identifier, BakedModel)> = vec![];

    for state in registry.get_blockstate_register().get_elements() {
        let identifier = state.get_state_identifier().get_identifier();
        let block_id = state.get_block_identifier();

        let blockstate_file = blockstate_files.get(block_id).expect(format!("Missing blockstate file for {}", block_id).as_str());
        let blockstate_model = match &blockstate_file {
            MCBlockstateType::variants(variants) => {
                let mut t_variant_model = Identifier::from_str(variants.get("").map(|v| v["model"].as_str().unwrap()).unwrap_or("minecraft:missing"));
                for (property_list, variant_model) in variants {
                    let mut valid_variant = true;
                    for variant_property in property_list.split(",") {
                        valid_variant &= identifier.contains(variant_property);
                    }
                    if valid_variant {
                        t_variant_model = Identifier::from(variant_model["model"].as_str().unwrap());
                    }
                }
                if !t_variant_model.get_name().starts_with("block/") {
                    t_variant_model = Identifier::new(t_variant_model.get_namespace().clone(), format!("block/{}", t_variant_model.get_name()));
                }
                let model = match voxel_models.get(&t_variant_model) {
                    Some(model_file) => {
                        log::info!("Using model {} for blockstate {}", t_variant_model, identifier);
                        model_file
                    },
                    None => {
                        log::error!("Invalid model {} for blockstate {}!", t_variant_model, identifier);
                        missing_model_file
                    }
                };
                model
            },
            MCBlockstateType::multipart(multiparts) => {
                missing_model_file
            }
        };
        // registry.get_model_register_mut().insert;
        mapped_models.push((state.get_state_identifier().clone(), blockstate_model.clone().bake()));
    }

    for mapped_model in mapped_models {
        registry.get_model_register_mut().insert(mapped_model.0, mapped_model.1);
    }
}
