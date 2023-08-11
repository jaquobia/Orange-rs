use crate::{block::{block_factory::BlockFactory, Block}, direction::Direction};

fn slab_cull(dir: Direction) -> bool {
    dir == Direction::Down
}

fn non_full_cull(_: Direction) -> bool {
    false
}

pub fn blocks() -> Vec<Block> {
        vec![
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
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("still_water")
                .hardness(100.0)
                .transparent(true)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("flowing_lava")
                .hardness(0.0)
                .transparent(true)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("still_lava")
                .hardness(100.0)
                .transparent(true)
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
                .transparent(false)
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
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("powered_rail")
                .hardness(0.7)
                .properties(&vec![("shape", "minecraft:rail_no_curve"), ("powered", "minecraft:boolean")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("detector_rail")
                .hardness(0.7)
                .properties(&vec![("shape", "minecraft:rail_no_curve"), ("powered", "minecraft:boolean")])
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
            BlockFactory::new("moving_piston")
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
            BlockFactory::new("double_slab") // double stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .properties(&vec![("type", "minecraft:slab_type")])
                .build(),
            BlockFactory::new("slab") // single stone slab block
                .hardness(2.0)
                .resistance(10.0)
                .properties(&vec![("type", "minecraft:slab_type")])
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
                .properties(&vec![("meta", "minecraft:count_4")])
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
                .properties(&vec![("stage", "minecraft:count_7")])
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
                .build(),
            BlockFactory::new("furnace_active")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .hardness(3.5)
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
                .properties(&vec![("shape", "minecraft:rail_with_curve")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("cobblestone_stairs")
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
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
                .properties(&vec![("powered", "minecraft:boolean")])
                .full_block(false)
                .build(),
            BlockFactory::new("iron_door")
                .hardness(3.0)
                .side_cull_fn(non_full_cull)
                .properties(&vec![("facing", "minecraft:facing_horizontal"), ("half", "minecraft:block_half"), ("powered", "minecraft:boolean")])
                .build(),
            BlockFactory::new("wooden_pressure_plate")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .properties(&vec![("powered", "minecraft:boolean")])
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
            BlockFactory::new("redstone_torch_off")
                .hardness(0.0)
                .properties(&vec![("meta", "minecraft:count_4")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("redstone_torch_on")
                .hardness(0.0)
                .properties(&vec![("meta", "minecraft:count_4")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("button")
                .hardness(0.5)
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("snow_layer") // TODO: Check for culling, seems to not cull other snow layers
                .hardness(0.1)
                .side_cull_fn(slab_cull)
                // .side_cull_fn(non_full_cull)
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
                .properties(&vec![("orientation", "minecraft:orientation_2d")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
            BlockFactory::new("pumpkin_lantern")
                .hardness(1.0)
                .properties(&vec![("facing", "minecraft:facing_horizontal")])
                .build(),
            BlockFactory::new("cake")
                .hardness(0.5)
                .properties(&vec![("slices", "minecraft:count_5")])
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
                .properties(&vec![("facing", "minecraft:facing_horizontal"), ("powered", "minecraft:boolean")])
                .side_cull_fn(non_full_cull)
                .full_block(false)
                .build(),
        ]
}
