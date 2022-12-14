pub mod block_factory;

use crate::{registry::Registerable, identifier::Identifier};

use self::block_factory::BlockSettings;

/// Describes the properties of blocks in the world, how they look, how they interact with
/// entities, and if they have an associated entity

pub struct Block {
    identifier: Identifier,
    hardness: f32,
    resistance: f32,
    slipperiness: f32,
    transparent: bool,
    texture_index: usize,
}

impl Block {
    pub fn new(identifier: Identifier, settings: BlockSettings) -> Self {
        let hardness = settings.hardness.unwrap_or(0.0);
        let hardness_5 = hardness * 5.0;
        let resistance = match settings.resistance {
            Some(res) => { 3.0 * res },
            None => { hardness_5 },
        };
        // For minecraft b1.7.3 functional parity, but seems to never really be used?
        let resistance = resistance.max(hardness_5);

        let slipperiness = settings.slipperiness.unwrap_or(0.0);

        let transparent = settings.transparent.unwrap_or(false);
        
        let texture_index = settings.texture_index.unwrap_or(0);
        Self {
            identifier,
            hardness,
            resistance,
            slipperiness,
            transparent,
            texture_index,
        }
    }
    
    pub fn get_hardness(&self) -> f32 {
        self.hardness
    }

    pub fn get_blast_resistance(&self) -> f32 {
        self.resistance
    }

    pub fn get_slipperiness(&self) -> f32 {
        self.slipperiness
    }

    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    pub fn texture_index(&self) -> usize {
        self.texture_index
    }
}

impl Registerable for Block {
    fn get_identifier(&self) -> &Identifier {
        return &self.identifier;
    }
}

impl Default for Block {
    fn default() -> Self {
        let identifier = Identifier::from("stone");
        let settings: BlockSettings = Default::default();
        Block::new(identifier, settings)
    }
}

pub struct BlockState {

}

impl BlockState {

}


