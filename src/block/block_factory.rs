use crate::block::SideCullFunctionType;
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

    pub fn full_block(mut self, f: bool) -> Self {
        self.settings.full_block = Some(f);
        self
    }

    pub fn properties(mut self, f: &[(&str, &str)]) -> Self {
        let mut vec: Vec<_> = f.into_iter().map(|&s| (s.0.to_string(), s.1.into())).collect();
        vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.settings.properties = Some(vec);
        self
    }

    pub fn side_cull_fn(mut self, f: SideCullFunctionType) -> Self {
        self.settings.side_cull_fn = Some(f);
        self
    }
}

#[derive(Clone, Default)]
pub struct BlockSettings {
    pub hardness: Option<f32>,
    pub resistance: Option<f32>,
    pub slipperiness: Option<f32>,
    pub transparent: Option<bool>,
    pub full_block: Option<bool>,
    pub properties: Option<Vec<(String, Identifier)>>,
    pub side_cull_fn: Option<SideCullFunctionType>,
}
