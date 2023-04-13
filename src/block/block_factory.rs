use crate::block::ModelSupplierType;
use crate::minecraft::identifier::Identifier;

use super::Block;

/** A functional style factory for making Blocks
 *  Start a new factory with BlockFactory::new(identifier)
 *  Construct a block with BlockFactory::build()
 */
pub struct BlockFactory {
    identifier: Identifier,
    settings: BlockSettings,
}

impl BlockFactory {
    pub fn new(into_identifier: impl Into<Identifier>) -> Self {
        let identifier: Identifier = into_identifier.into();
        Self {
            identifier,
            settings: Default::default(),
        }
    }

    /** Build the block from the stored data
     *
     */
    pub fn build(self) -> Block {
        Block::new(self.identifier, self.settings)
    }

    pub fn hardness(mut self, f: f32) -> Self {
        self.settings.hardness = Some(f);
        self
    }

    pub fn resistance(mut self, f: f32) -> Self {
        self.settings.resistance = Some(f);
        self
    }

    pub fn slipperiness(mut self, f: f32) -> Self {
        self.settings.slipperiness = Some(f);
        self
    }

    pub fn transparent(mut self, f: bool) -> Self {
        self.settings.transparent = Some(f);
        self
    }

    pub fn texture_index(mut self, f: usize) -> Self {
        self.settings.texture_index = Some(f);
        self
    }

    pub fn model(mut self, f: ModelSupplierType) -> Self {
        self.settings.model_supplier = Some(f);
        self
    }
}

#[derive(Clone, Copy, Default)]
pub struct BlockSettings {
    pub hardness: Option<f32>,
    pub resistance: Option<f32>,
    pub slipperiness: Option<f32>,
    pub transparent: Option<bool>,
    pub texture_index: Option<usize>,
    pub model_supplier: Option<ModelSupplierType>,
}
