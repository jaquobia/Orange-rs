use log::warn;
use ultraviolet::{Vec2, Vec3};
use crate::{registry::Registry, block::block_factory::BlockFactory};
use crate::client::models::model::{BakedModel, VoxelElement, VoxelFace, VoxelModel, VoxelRotation};
use crate::client::textures::TextureObject;
use crate::client::textures::TextureObject::AtlasTexture;
use crate::direction::Direction;
use crate::minecraft::identifier::Identifier;
use crate::minecraft::template_models;
use crate::minecraft::template_models::{column, column_top_bottom, crop, cross, cube, cube_all, door_bottom_left, door_bottom_right, door_top_left, door_top_right, missing, orientable, pressure_plate_down, pressure_plate_up, stair_all, torch, wall_torch};

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

fn load_b173(registry: &mut Registry) {

    let textures = registry.get_texture_register_mut();
    textures.insert(Identifier::from("minecraft:grass_top"), make_atlas_tex(0));
    textures.insert(Identifier::from("minecraft:stone"), make_atlas_tex(1));
    textures.insert(Identifier::from("minecraft:dirt"), make_atlas_tex(2));
    textures.insert(Identifier::from("minecraft:grass_side"), make_atlas_tex(3));
    textures.insert(Identifier::from("minecraft:oak_plank"), make_atlas_tex(4));
    textures.insert(Identifier::from("minecraft:stone_slab_side"), make_atlas_tex(5));
    textures.insert(Identifier::from("minecraft:stone_slab_top"), make_atlas_tex(6));
    textures.insert(Identifier::from("minecraft:bricks"), make_atlas_tex(7));
    textures.insert(Identifier::from("minecraft:tnt_side"), make_atlas_tex(8));
    textures.insert(Identifier::from("minecraft:tnt_top"), make_atlas_tex(9));
    textures.insert(Identifier::from("minecraft:tnt_bottom"), make_atlas_tex(10));
    textures.insert(Identifier::from("minecraft:cobweb"), make_atlas_tex(11));
    textures.insert(Identifier::from("minecraft:red_flower"), make_atlas_tex(12));
    textures.insert(Identifier::from("minecraft:yellow_flower"), make_atlas_tex(13));
    textures.insert(Identifier::from("minecraft:portal"), make_atlas_tex(14));
    textures.insert(Identifier::from("minecraft:oak_sapling"), make_atlas_tex(15));
    textures.insert(Identifier::from("minecraft:cobblestone"), make_atlas_tex(16));
    textures.insert(Identifier::from("minecraft:bedrock"), make_atlas_tex(17));
    textures.insert(Identifier::from("minecraft:sand"), make_atlas_tex(18));
    textures.insert(Identifier::from("minecraft:gravel"), make_atlas_tex(19));
    textures.insert(Identifier::from("minecraft:oak_log_side"), make_atlas_tex(20));
    textures.insert(Identifier::from("minecraft:oak_log_top"), make_atlas_tex(21));
    textures.insert(Identifier::from("minecraft:iron_block"), make_atlas_tex(22));
    textures.insert(Identifier::from("minecraft:gold_block"), make_atlas_tex(23));
    textures.insert(Identifier::from("minecraft:diamond_block"), make_atlas_tex(24));
    textures.insert(Identifier::from("minecraft:chest_top"), make_atlas_tex(25));
    textures.insert(Identifier::from("minecraft:chest_side_single"), make_atlas_tex(26));
    textures.insert(Identifier::from("minecraft:chest_front"), make_atlas_tex(27));
    textures.insert(Identifier::from("minecraft:red_mushroom"), make_atlas_tex(28));
    textures.insert(Identifier::from("minecraft:brown_mushroom"), make_atlas_tex(29));
    textures.insert(Identifier::from("missing"), make_atlas_tex(30));
    // 30
    textures.insert(Identifier::from("minecraft:fire_1"), make_atlas_tex(31));
    textures.insert(Identifier::from("minecraft:gold_ore"), make_atlas_tex(32));
    textures.insert(Identifier::from("minecraft:iron_ore"), make_atlas_tex(33));
    textures.insert(Identifier::from("minecraft:coal_ore"), make_atlas_tex(34));
    textures.insert(Identifier::from("minecraft:bookshelf"), make_atlas_tex(35));
    textures.insert(Identifier::from("minecraft:mossy_cobblestone"), make_atlas_tex(36));
    textures.insert(Identifier::from("minecraft:obsidian"), make_atlas_tex(37));
    textures.insert(Identifier::from("minecraft:grass_side_overlay"), make_atlas_tex(38));
    textures.insert(Identifier::from("minecraft:tall_grass"), make_atlas_tex(39));
    textures.insert(Identifier::from("minecraft:grass_top_2"), make_atlas_tex(40));
    textures.insert(Identifier::from("minecraft:chest_front_double_left"), make_atlas_tex(41));
    textures.insert(Identifier::from("minecraft:chest_front_double_right"), make_atlas_tex(42));
    textures.insert(Identifier::from("minecraft:workbench_top"), make_atlas_tex(43));
    textures.insert(Identifier::from("minecraft:furnace_front"), make_atlas_tex(44));
    textures.insert(Identifier::from("minecraft:furnace_side"), make_atlas_tex(45));
    textures.insert(Identifier::from("minecraft:dispenser_front"), make_atlas_tex(46));
    textures.insert(Identifier::from("minecraft:fire_2"), make_atlas_tex(47));
    textures.insert(Identifier::from("minecraft:sponge"), make_atlas_tex(48));
    textures.insert(Identifier::from("minecraft:glass"), make_atlas_tex(49));
    textures.insert(Identifier::from("minecraft:diamond_ore"), make_atlas_tex(50));
    textures.insert(Identifier::from("minecraft:redstone_ore"), make_atlas_tex(51));
    textures.insert(Identifier::from("minecraft:leaves_oak"), make_atlas_tex(52));
    textures.insert(Identifier::from("minecraft:leaves_oak_2"), make_atlas_tex(53));
    // 54
    textures.insert(Identifier::from("minecraft:dead_bush"), make_atlas_tex(55));
    textures.insert(Identifier::from("minecraft:fern"), make_atlas_tex(56));
    textures.insert(Identifier::from("minecraft:chest_side_double_left"), make_atlas_tex(57));
    textures.insert(Identifier::from("minecraft:chest_side_double_right"), make_atlas_tex(58));
    textures.insert(Identifier::from("minecraft:workbench_front"), make_atlas_tex(59));
    textures.insert(Identifier::from("minecraft:workbench_side"), make_atlas_tex(60));
    textures.insert(Identifier::from("minecraft:furnace_front_lit"), make_atlas_tex(61));
    textures.insert(Identifier::from("minecraft:furnace_top"), make_atlas_tex(62));
    textures.insert(Identifier::from("minecraft:spruce_sapling"), make_atlas_tex(63));
    textures.insert(Identifier::from("minecraft:white_wool"), make_atlas_tex(64));
    textures.insert(Identifier::from("minecraft:mob_spawner"), make_atlas_tex(65));
    textures.insert(Identifier::from("minecraft:snow"), make_atlas_tex(66));
    textures.insert(Identifier::from("minecraft:ice"), make_atlas_tex(67));
    textures.insert(Identifier::from("minecraft:grass_side_snowy"), make_atlas_tex(68));
    textures.insert(Identifier::from("minecraft:cactus_top"), make_atlas_tex(69));
    textures.insert(Identifier::from("minecraft:cactus_side"), make_atlas_tex(70));
    textures.insert(Identifier::from("minecraft:cactus_bottom"), make_atlas_tex(71));
    textures.insert(Identifier::from("minecraft:clay"), make_atlas_tex(72));
    textures.insert(Identifier::from("minecraft:reed"), make_atlas_tex(73));
    textures.insert(Identifier::from("minecraft:jukebox_side"), make_atlas_tex(74));
    textures.insert(Identifier::from("minecraft:jukebox_top"), make_atlas_tex(75));
    // 76
    // 77
    // 78
    textures.insert(Identifier::from("minecraft:birch_sapling"), make_atlas_tex(79));
    textures.insert(Identifier::from("minecraft:torch"), make_atlas_tex(80));
    textures.insert(Identifier::from("minecraft:oak_door_top"), make_atlas_tex(81));
    textures.insert(Identifier::from("minecraft:iron_door_top"), make_atlas_tex(82));
    textures.insert(Identifier::from("minecraft:ladder"), make_atlas_tex(83));
    textures.insert(Identifier::from("minecraft:oak_trap_door"), make_atlas_tex(84));
    // 85
    textures.insert(Identifier::from("minecraft:farmland_wet"), make_atlas_tex(86));
    textures.insert(Identifier::from("minecraft:farmland_dry"), make_atlas_tex(87));
    textures.insert(Identifier::from("minecraft:wheat_0"), make_atlas_tex(88));
    textures.insert(Identifier::from("minecraft:wheat_1"), make_atlas_tex(89));
    textures.insert(Identifier::from("minecraft:wheat_2"), make_atlas_tex(90));
    textures.insert(Identifier::from("minecraft:wheat_3"), make_atlas_tex(91));
    textures.insert(Identifier::from("minecraft:wheat_4"), make_atlas_tex(92));
    textures.insert(Identifier::from("minecraft:wheat_5"), make_atlas_tex(93));
    textures.insert(Identifier::from("minecraft:wheat_6"), make_atlas_tex(94));
    textures.insert(Identifier::from("minecraft:wheat_7"), make_atlas_tex(95));
    textures.insert(Identifier::from("minecraft:lever"), make_atlas_tex(96));
    textures.insert(Identifier::from("minecraft:oak_door_bottom"), make_atlas_tex(97));
    textures.insert(Identifier::from("minecraft:iron_door_bottom"), make_atlas_tex(98));
    textures.insert(Identifier::from("minecraft:redstone_torch_on"), make_atlas_tex(99));
    // 100
    // 101
    textures.insert(Identifier::from("minecraft:pumpkin_top"), make_atlas_tex(102));
    textures.insert(Identifier::from("minecraft:netherrack"), make_atlas_tex(103));
    textures.insert(Identifier::from("minecraft:soul_sand"), make_atlas_tex(104));
    textures.insert(Identifier::from("minecraft:glowstone"), make_atlas_tex(105));
    textures.insert(Identifier::from("minecraft:piston_sticky_front"), make_atlas_tex(106));
    textures.insert(Identifier::from("minecraft:piston_front"), make_atlas_tex(107));
    textures.insert(Identifier::from("minecraft:piston_side"), make_atlas_tex(108));
    textures.insert(Identifier::from("minecraft:piston_bottom"), make_atlas_tex(109));
    textures.insert(Identifier::from("minecraft:piston_base"), make_atlas_tex(110));
    // 111
    textures.insert(Identifier::from("minecraft:rail_curved"), make_atlas_tex(112));
    textures.insert(Identifier::from("minecraft:black_wool"), make_atlas_tex(113));
    textures.insert(Identifier::from("minecraft:grey_wool"), make_atlas_tex(114));
    textures.insert(Identifier::from("minecraft:redstone_torch_off"), make_atlas_tex(115));
    textures.insert(Identifier::from("minecraft:spruce_log_side"), make_atlas_tex(116));
    textures.insert(Identifier::from("minecraft:birch_log_side"), make_atlas_tex(117));
    textures.insert(Identifier::from("minecraft:pumpkin_side"), make_atlas_tex(118));
    textures.insert(Identifier::from("minecraft:pumpkin_front"), make_atlas_tex(119));
    textures.insert(Identifier::from("minecraft:lantern_front"), make_atlas_tex(120));
    textures.insert(Identifier::from("minecraft:cake_top"), make_atlas_tex(121));
    textures.insert(Identifier::from("minecraft:cake_side"), make_atlas_tex(122));
    textures.insert(Identifier::from("minecraft:cake_side_eaten"), make_atlas_tex(123));
    textures.insert(Identifier::from("minecraft:cake_bottom"), make_atlas_tex(124));
    // 125
    // 126
    // 127
    textures.insert(Identifier::from("minecraft:rail"), make_atlas_tex(128));
    textures.insert(Identifier::from("minecraft:red_wool"), make_atlas_tex(129));
    textures.insert(Identifier::from("minecraft:pink_wool"), make_atlas_tex(130));
    textures.insert(Identifier::from("minecraft:repeater_top_off"), make_atlas_tex(131));
    textures.insert(Identifier::from("minecraft:leaves_spruce"), make_atlas_tex(132));
    textures.insert(Identifier::from("minecraft:leaves_spruce_2"), make_atlas_tex(133));
    textures.insert(Identifier::from("minecraft:bed_top_foot"), make_atlas_tex(134));
    textures.insert(Identifier::from("minecraft:bed_top_head"), make_atlas_tex(135));
    // 136
    // 137
    // 138
    // 139
    textures.insert(Identifier::from("minecraft:cake_item"), make_atlas_tex(140));
    // 141
    // 142
    // 143
    textures.insert(Identifier::from("minecraft:lapis_block"), make_atlas_tex(144));
    textures.insert(Identifier::from("minecraft:green_wool"), make_atlas_tex(145));
    textures.insert(Identifier::from("minecraft:lime_wool"), make_atlas_tex(146));
    textures.insert(Identifier::from("minecraft:repeater_top_on"), make_atlas_tex(147));
    // 148
    textures.insert(Identifier::from("minecraft:bed_side_foot_front"), make_atlas_tex(149));
    textures.insert(Identifier::from("minecraft:bed_side_foot"), make_atlas_tex(150));
    textures.insert(Identifier::from("minecraft:bed_side_head"), make_atlas_tex(151));
    textures.insert(Identifier::from("minecraft:bed_side_head_front"), make_atlas_tex(152));
    // 153 - 159
    textures.insert(Identifier::from("minecraft:lapis_ore"), make_atlas_tex(160));
    textures.insert(Identifier::from("minecraft:brown_wool"), make_atlas_tex(161));
    textures.insert(Identifier::from("minecraft:yellow_wool"), make_atlas_tex(162));
    textures.insert(Identifier::from("minecraft:powered_rail_off"), make_atlas_tex(163));
    textures.insert(Identifier::from("minecraft:redstone_dust"), make_atlas_tex(164));
    textures.insert(Identifier::from("minecraft:redstone_dust_line"), make_atlas_tex(165));
    // 166 - 175
    textures.insert(Identifier::from("minecraft:sandstone_top"), make_atlas_tex(176));
    textures.insert(Identifier::from("minecraft:blue_wool"), make_atlas_tex(177));
    textures.insert(Identifier::from("minecraft:light_blue_wool"), make_atlas_tex(178));
    textures.insert(Identifier::from("minecraft:powered_rail_on"), make_atlas_tex(179));
    // 180 - 191
    textures.insert(Identifier::from("minecraft:sandstone_side"), make_atlas_tex(192));
    textures.insert(Identifier::from("minecraft:purple_wool"), make_atlas_tex(193));
    textures.insert(Identifier::from("minecraft:magenta_wool"), make_atlas_tex(194));
    textures.insert(Identifier::from("minecraft:detector_rail"), make_atlas_tex(195));
    // 196 - 204
    textures.insert(Identifier::from("minecraft:water_0"), make_atlas_tex(205));
    textures.insert(Identifier::from("minecraft:water_1"), make_atlas_tex(206));
    textures.insert(Identifier::from("minecraft:water_2"), make_atlas_tex(207));
    textures.insert(Identifier::from("minecraft:sandstone_bottom"), make_atlas_tex(208));
    textures.insert(Identifier::from("minecraft:cyan_wool"), make_atlas_tex(209));
    textures.insert(Identifier::from("minecraft:orange_wool"), make_atlas_tex(210));
    // 211 - 221
    textures.insert(Identifier::from("minecraft:water_3"), make_atlas_tex(222));
    textures.insert(Identifier::from("minecraft:water_4"), make_atlas_tex(223));
    // 224
    textures.insert(Identifier::from("minecraft:light_grey_wool"), make_atlas_tex(225));
    // 226 - 236
    textures.insert(Identifier::from("minecraft:lava_0"), make_atlas_tex(237));
    textures.insert(Identifier::from("minecraft:lava_1"), make_atlas_tex(238));
    textures.insert(Identifier::from("minecraft:lava_2"), make_atlas_tex(239));
    textures.insert(Identifier::from("minecraft:break_0"), make_atlas_tex(240));
    textures.insert(Identifier::from("minecraft:break_1"), make_atlas_tex(241));
    textures.insert(Identifier::from("minecraft:break_2"), make_atlas_tex(242));
    textures.insert(Identifier::from("minecraft:break_3"), make_atlas_tex(243));
    textures.insert(Identifier::from("minecraft:break_4"), make_atlas_tex(244));
    textures.insert(Identifier::from("minecraft:break_5"), make_atlas_tex(245));
    textures.insert(Identifier::from("minecraft:break_6"), make_atlas_tex(246));
    textures.insert(Identifier::from("minecraft:break_7"), make_atlas_tex(247));
    textures.insert(Identifier::from("minecraft:break_8"), make_atlas_tex(248));
    textures.insert(Identifier::from("minecraft:break_9"), make_atlas_tex(249));
    // 250 - 253
    textures.insert(Identifier::from("minecraft:lava_3"), make_atlas_tex(254));
    textures.insert(Identifier::from("minecraft:lava_4"), make_atlas_tex(255));


    // Blocks & Items
    let blocks = registry.get_block_register_mut();
    let block_register_list = vec![
            BlockFactory::new("air")
                .hardness(0.0)
                .resistance(0.0)
                .model(|_| {
                        BakedModel::new()
                })
                .transparent(true)
                .full_block(false)
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("stone")
                .hardness(1.5)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:stone").bake()
                })
                .build(),
            BlockFactory::new("grass")
                .hardness(0.6)
                .model(|_| {
                    // TODO: SNOW VARIANT, requires surrounding block information
                    VoxelModel::new()
                        .with_texture("particle", "minecraft:dirt")
                        .with_texture("top", "minecraft:grass_top")
                        .with_texture("bottom", "minecraft:dirt")
                        .with_texture("side", "minecraft:grass_side")
                        .with_texture("overlay", "minecraft:grass_side_overlay")
                        .with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0])
                            .with_face(VoxelFace::new("#top").with_cullface(Direction::Up), Direction::Up)
                            .with_face(VoxelFace::new("#bottom").with_cullface(Direction::Down), Direction::Down)
                            .with_face(VoxelFace::new("#side").with_cullface(Direction::North), Direction::North)
                            .with_face(VoxelFace::new("#side").with_cullface(Direction::South), Direction::South)
                            .with_face(VoxelFace::new("#side").with_cullface(Direction::East), Direction::East)
                            .with_face(VoxelFace::new("#side").with_cullface(Direction::West), Direction::West)
                        ).with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 16.0, 16.0])
                        .with_face(VoxelFace::new("#overlay").with_cullface(Direction::North), Direction::North)
                        .with_face(VoxelFace::new("#overlay").with_cullface(Direction::South), Direction::South)
                        .with_face(VoxelFace::new("#overlay").with_cullface(Direction::East), Direction::East)
                        .with_face(VoxelFace::new("#overlay").with_cullface(Direction::West), Direction::West)
                    ).bake()
                })
                .build(),
            BlockFactory::new("dirt")
                .hardness(0.5)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:dirt").bake()
                })
                .build(),
            BlockFactory::new("cobblestone")
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:cobblestone").bake()
                })
                .build(),
            BlockFactory::new("wood")
                .hardness(2.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:oak_plank").bake()
                })
                .build(),
            BlockFactory::new("sapling")
                .hardness(0.0)
                .model(|meta| {
                    match meta & 3 {
                        0 => VoxelModel::from_template(cross()).with_texture("cross", "minecraft:oak_sapling").bake(),
                        1 => VoxelModel::from_template(cross()).with_texture("cross", "minecraft:spruce_sapling").bake(),
                        2 => VoxelModel::from_template(cross()).with_texture("cross", "minecraft:birch_sapling").bake(),
                        _ => VoxelModel::from_template(missing()).bake(),
                    }
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("bedrock")
                .hardness(-1.0)
                .resistance(6000000.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:bedrock").bake()
                })
                .build(),
            BlockFactory::new("flowing_water")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:water_0").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("still_water")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:water_0").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("flowing_lava")
                .hardness(0.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:lava_0").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("still_lava")
                .hardness(100.0)
                .transparent(true)
                .model(|meta| {
                    // TODO: SUPER COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:lava_0").bake()
                })
                .full_block(false)
                .build(),
            BlockFactory::new("sand")
                .hardness(0.5)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:sand").bake()
                })
                .build(),
            BlockFactory::new("gravel")
                .hardness(0.5)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:gravel").bake()
                })
                .build(),
            BlockFactory::new("ore_gold")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:gold_ore").bake()
                })
                .build(),
            BlockFactory::new("ore_iron")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:iron_ore").bake()
                })
                .build(),
            BlockFactory::new("ore_coal")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:coal_ore").bake()
                })
                .build(),
            BlockFactory::new("wood")
                .hardness(2.0)
                .model(|meta| {
                    match meta {
                        0 => column().clone()
                            .with_texture("side", "minecraft:oak_log_side")
                            .with_texture("up", "minecraft:oak_log_top")
                            .bake(),
                        1 => column().clone()
                            .with_texture("side", "minecraft:spruce_log_side")
                            .with_texture("up", "minecraft:oak_log_top")
                            .bake(),
                        2 => column().clone()
                            .with_texture("side", "minecraft:birch_log_side")
                            .with_texture("up", "minecraft:oak_log_top")
                            .bake(),
                        _ => missing().clone().bake(),
                    }
                })
                .build(),
            BlockFactory::new("leaves")
                .hardness(0.2)
                .model(|meta| {
                    // TODO: Account for tint
                    let texture = match meta & 3 {
                        0 => "minecraft:leaves_oak",
                        1 => "minecraft:leaves_spruce",
                        2 => "minecraft:leaves_oak",
                        _ => return missing().clone().bake(),
                    };
                    cube_all().clone().with_texture("all", texture).bake()
                })
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("sponge")
                .hardness(0.6)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:sponge").bake()
                })
                .build(),
            BlockFactory::new("glass")
                .hardness(0.3)
                .resistance(6000000.0)
                // .transparent(true)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:glass").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("ore_lapis")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:lapis_ore").bake()
                })
                .build(),
            BlockFactory::new("block_lapis")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:lapis_block").bake()
                })
                .build(),
            BlockFactory::new("dispenser")
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
                        .with_texture("front", "minecraft:dispenser_front")
                        .with_texture("side", "minecraft:furnace_side")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("sandstone")
                .hardness(0.8)
                .model(|_| {
                    column_top_bottom().clone()
                        .with_texture("up", "minecraft:sandstone_top")
                        .with_texture("down", "minecraft:sandstone_bottom")
                        .with_texture("side", "minecraft:sandstone_side")
                        .bake()
                })
                .build(),
            BlockFactory::new("noteblock")
                .hardness(0.8)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:jukebox_side").bake()
                })
                .build(),
            BlockFactory::new("bed")
                .hardness(0.2)
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
                .model(|meta| {
                    // TODO: COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:powered_rail_off").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("detector_rail")
                .hardness(0.7)
                .model(|meta| {
                    // TODO: COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:detector_rail").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("sticky_piston")
                .model(|meta| {
                    // TODO: COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:piston_sticky_front").bake()
                })
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("web")
                .hardness(4.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:cobweb").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("tall_grass")
                .hardness(0.0)
                .model(|meta| {
                    let texture =
                        if meta == 0 { "minecraft:dead_bush" }
                        else if meta == 2 { "minecraft:fern" }
                        else { "minecraft:tall_grass" };
                    cross().clone().with_texture("cross", texture).bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("dead_bush")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:dead_bush").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("piston")
                .model(|meta| {
                    warn!("piston meta: {meta}");
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
                    warn!("Piston model end");
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
                .model(|meta| {
                    // TODO: COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:piston_front").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("wool")
                .hardness(0.8)
                .model(|meta| {
                    let texture = match meta {
                        0 => "minecraft:white_wool",
                        1 => "minecraft:orange_wool",
                        2 => "minecraft:magenta_wool",
                        3 => "minecraft:light_blue_wool",
                        4 => "minecraft:yellow_wool",
                        5 => "minecraft:lime_wool",
                        6 => "minecraft:pink_wool",
                        7 => "minecraft:grey_wool",
                        8 => "minecraft:light_grey_wool",
                        9 => "minecraft:cyan_wool",
                        10 => "minecraft:purple_wool",
                        11 => "minecraft:blue_wool",
                        12 => "minecraft:brown_wool",
                        13 => "minecraft:green_wool",
                        14 => "minecraft:red_wool",
                        15 => "minecraft:black_wool",
                        _ => "minecraft:missing",
                    };
                    cube_all().clone().with_texture("all", texture).bake()
                })
                .build(),
            BlockFactory::new("piston_moving")
                .hardness(-1.0)
                .model(|meta| {
                    // TODO: COMPLEX MODEL
                    cube_all().clone().with_texture("all", "minecraft:piston_front").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("yellow_flower")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:yellow_flower").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("red_flower")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:red_flower").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("brown_mushroom")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:brown_mushroom").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("red_mushroom")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:red_mushroom").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("block_gold")
                .hardness(3.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:gold_block").bake()
                })
                .build(),
            BlockFactory::new("block_iron")
                .hardness(5.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:iron_wool").bake()
                })
                .build(),
            BlockFactory::new("double_stair") // double stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    column().clone()
                        .with_texture("up", "minecraft:stone_slab_top")
                        .with_texture("side", "minecraft:stone_slab_side")
                        .bake()
                })
                .build(),
            BlockFactory::new("single_stair") // single stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    VoxelModel::from_template(template_models::slab_column())
                        .with_texture("up", "minecraft:stone_slab_top")
                        .with_texture("side", "minecraft:stone_slab_side")
                        .bake()
                })
                .side_cull_fn(slab_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("brick_block")
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:bricks").bake()
                })
                .build(),
            BlockFactory::new("tnt")
                .hardness(0.0)
                .model(|_| {
                    column_top_bottom().clone()
                        .with_texture("up", "minecraft:tnt_top")
                        .with_texture("down", "minecraft:tnt_bottom")
                        .with_texture("side", "minecraft:tnt_side")
                        .bake()
                })
                .build(),
            BlockFactory::new("bookshelf")
                .hardness(1.5)
                .model(|_| {
                    column().clone()
                        .with_texture("up", "minecraft:oak_plank")
                        .with_texture("side", "minecraft:bookshelf")
                        .bake()
                })
                .build(),
            BlockFactory::new("mossy_cobblestone")
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:mossy_cobblestone").bake()
                })
                .build(),
            BlockFactory::new("obsidian")
                .hardness(10.0)
                .resistance(2000.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:obsidian").bake()
                })
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
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:mob_spawner").bake()
                })
                .build(),
            BlockFactory::new("wooden_stairs")
                .model(|meta| {
                    let rotation = match meta {
                        0 => 0.,
                        1 => 180.,
                        2 => 270.,
                        3 => 90.,
                        _ => return missing().clone().bake(),
                    };
                    stair_all().clone().with_texture("all", "minecraft:oak_plank")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("chest")
                .hardness(2.5)
                .model(|_| {
                    // TODO: Account for nearby blocks for rotation, requires surrounding block information
                    let rotation = 90.;
                    orientable().clone()
                        .with_texture("up", "minecraft:chest_top")
                        .with_texture("down", "minecraft:chest_top")
                        .with_texture("front", "minecraft:chest_front")
                        .with_texture("side", "minecraft:chest_side_single")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("redstone_dust")
                .hardness(0.0)
                // TODO: COMPLEX MODEL
                .full_block(false)
                .build(),
            BlockFactory::new("ore_diamond")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:diamond_ore").bake()
                })
                .build(),
            BlockFactory::new("block_diamond")
                .hardness(5.0)
                .resistance(10.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:block_diamond").bake()
                })
                .build(),
            BlockFactory::new("workbench")
                .hardness(2.5)
                .model(|_| cube().clone()
                    .with_texture("up", "minecraft:workbench_top")
                    .with_texture("north", "minecraft:workbench_side")
                    .with_texture("east", "minecraft:workbench_side")
                    .with_texture("south", "minecraft:workbench_front")
                    .with_texture("west", "minecraft:workbench_front")
                    .with_texture("down", "minecraft:oak_plank")
                    .bake()
                )
                .build(),
            BlockFactory::new("crops")
                .hardness(0.0)
                .model(|meta| {
                    let tex_id = format!("minecraft:wheat_{}", meta);
                    crop().clone()
                        .with_texture("crop", tex_id)
                        .bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("tilled_dirt")
                .hardness(0.6)
                .model(|meta| {
                    let top_tex_id = if meta > 0 { "minecraft:farmland_wet" } else { "minecraft:farmland_dry" };
                    VoxelModel::new().with_element(VoxelElement::new([0.0, 0.0, 0.0], [16.0, 15.0, 16.0])
                       .with_face(VoxelFace::new("#side").with_cullface(Direction::Down), Direction::Down)
                       .with_face(VoxelFace::new(top_tex_id), Direction::Up)
                       .with_face(VoxelFace::new("#side").with_uv([0., 1.], [16., 16.]).with_cullface(Direction::North), Direction::North)
                       .with_face(VoxelFace::new("#side").with_uv([0., 1.], [16., 16.]).with_cullface(Direction::South), Direction::South)
                       .with_face(VoxelFace::new("#side").with_uv([0., 1.], [16., 16.]).with_cullface(Direction::West), Direction::West)
                       .with_face(VoxelFace::new("#side").with_uv([0., 1.], [16., 16.]).with_cullface(Direction::East), Direction::East)
                    ).with_texture("side", "minecraft:dirt").bake()
                })
                .side_cull_fn(|dir| dir == Direction::Down)
                .build(),
            BlockFactory::new("furnace")
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
                .model(|meta| {
                    let rotation = match meta {
                        2 => 0.,
                        3 => 180.,
                        4 => 90.,
                        5 => 270.,
                        _ => 0.,
                    };
                    let voxel_rotation = Some(VoxelRotation::new(rotation, 1, [8., 8., 8.], false));
                    VoxelModel::new().with_element(VoxelElement::new([0., 0., 15.2], [16., 16., 15.2])
                        .with_face(VoxelFace::new("#ladder"), Direction::East)
                    ).with_texture("ladder", "minecraft:ladder").bake_with_rotate(voxel_rotation)
                })
                .full_block(false)
                .build(),
            BlockFactory::new("rail")
                .hardness(0.7)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("cobblestone_stair")
                .hardness(3.0)
                .model(|_| {
                    stair_all().clone().with_texture("all", "minecraft:cobblestone").bake()
                })
                .build(),
            BlockFactory::new("wall_sign")
                .hardness(1.0)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("lever")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                // TODO: Simple Model?
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
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:redstone_ore").bake()
                })
                .build(),
            BlockFactory::new("ore_redstone_glowing")
                .hardness(3.0)
                .resistance(5.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:redstone_ore").bake()
                })
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
                // TODO: Simple Model?
                .full_block(false)
                .build(),
            BlockFactory::new("snow_layer")
                .hardness(0.1)
                .side_cull_fn(slab_cull)
                // TODO: Simple Model?
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("ice")
                .hardness(0.5)
                .transparent(true)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:ice").bake()
                })
                // .side_cull_fn(non_full_cull)
                .build(),
            BlockFactory::new("snow")
                .hardness(0.2)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:snow").bake()
                })
                .build(),
            BlockFactory::new("cactus")
                .hardness(0.4)
                .side_cull_fn(non_full_cull)
                // TODO: Simple Model?
                .full_block(false)
                .build(),
            BlockFactory::new("clay_block")
                .hardness(0.6)
                .resistance(6000000.0)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:clay").bake()
                })
                .build(),
            BlockFactory::new("reed")
                .hardness(0.0)
                .model(|_| {
                    cross().clone().with_texture("cross", "minecraft:reed").bake()
                })
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("jukebox")
                .hardness(2.0)
                .resistance(10.0)
                .model(|_| {
                    cube().clone()
                        .with_texture("up", "minecraft:jukebox_top")
                        .with_texture("down", "minecraft:jukebox_side")
                        .with_texture("north", "minecraft:jukebox_side")
                        .with_texture("south", "minecraft:jukebox_side")
                        .with_texture("east", "minecraft:jukebox_side")
                        .with_texture("west", "minecraft:jukebox_side").bake()
                })
                .build(),
            BlockFactory::new("fence")
                .hardness(2.0)
                .resistance(5.0)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("pumpkin")
                .hardness(1.0)
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
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:netherrack").bake()
                })
                .build(),
            BlockFactory::new("soulsand")
                .hardness(0.5)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:soulsand").bake()
                })
                .build(),
            BlockFactory::new("glowstone_block")
                .hardness(0.3)
                .model(|_| {
                    cube_all().clone().with_texture("all", "minecraft:glowstone").bake()
                })
                .build(),
            BlockFactory::new("portal")
                .hardness(-1.0)
                .transparent(true)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("pumpkin_lantern")
                .hardness(1.0)
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
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("repeater_off")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("repeater_on")
                .hardness(0.0)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
            BlockFactory::new("locked_chest")
                .hardness(0.0)
                .model(|_| {
                    // TODO: Account for nearby blocks for rotation, requires surrounding block information
                    let rotation = 90.;
                    orientable().clone()
                        .with_texture("up", "minecraft:chest_top")
                        .with_texture("down", "minecraft:chest_top")
                        .with_texture("front", "minecraft:chest_front")
                        .with_texture("side", "minecraft:chest_side_single")
                        .bake_with_rotate(Some(VoxelRotation::new(rotation, 1, [8.0, 8.0, 8.0], false)))
                })
                .build(),
            BlockFactory::new("trapdoor")
                .hardness(-1.0)
                .side_cull_fn(non_full_cull)
                // TODO: Complex Model
                .full_block(false)
                .build(),
        ];

        for block in block_register_list {
            blocks.insert(block);
        }

    // Items


    // Recipes
    

    // Dimensions
    


}
