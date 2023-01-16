use crate::registry::Registry;

pub enum GameVersion {
    B173,
    /// Possibly use for stationapi comapt, which is b173 + modding capabilities
    STATION,
}

impl GameVersion {
    pub fn load_registry(&self, registry: &mut Registry) {
        match self {
            B173 => load_b173(registry),
            _ => {},
        }
    }
}

fn load_b173(registry: &mut Registry) {
    // Blocks & Items
    let blocks = registry.get_block_register_mut();
    let block_register_list = vec![
            BlockFactory::new("air")
                .hardness(0.0)
                .resistance(0.0)
                .texture_index(0)
                .transparent(true)
                .build(),
            BlockFactory::new("stone")
                .hardness(1.5)
                .resistance(10.0)
                .texture_index(1)
                .build(),
            BlockFactory::new("grass")
                .hardness(0.6)
                .texture_index(3)
                .build(),
            BlockFactory::new("dirt")
                .hardness(0.5)
                .texture_index(2)
                .build(),
            BlockFactory::new("cobblestone")
                .hardness(2.0)
                .resistance(10.0)
                .texture_index(17)
                .build(),
            BlockFactory::new("wood")
                .hardness(2.0)
                .resistance(5.0)
                .texture_index(4)
                .build(),
            BlockFactory::new("sapling")
                .hardness(0.0)
                .transparent(true)
                .texture_index(15)
                .build(),
            BlockFactory::new("bedrock")
                .hardness(-1.0)
                .resistance(6000000.0)
                .texture_index(17)
                .build(),
        ];

        for block in block_register_list {
            blocks.insert(block);
        }

    // Items


    // Recipes
    

    // Dimensions
    


}
