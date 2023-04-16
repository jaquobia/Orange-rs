pub mod block_factory;

use crate::{minecraft::identifier::Identifier, registry::Registerable};
use crate::client::models::model::{BakedModel, VoxelModel};
use crate::direction::Direction;
use crate::minecraft::template_models;

use self::block_factory::BlockSettings;

pub type ModelSupplierType = fn(u32) -> BakedModel;
pub type SideCullFunctionType = fn(Direction) -> bool;

/// Describes the properties of blocks in the world, how they look, how they interact with
/// entities, and if they have an associated entity

pub struct Block {
    identifier: Identifier,
    /// Hardness, determines the mining speed of the block
    hardness: f32,
    /// Blast Resistance, determines how effective explosions are against this block
    resistance: f32,
    /// Slipperiness, determines if the player will slide on this block and how fast (or slow)
    slipperiness: f32,
    /// Transparent, determines if this block should be on the transparency layer
    transparent: bool,

    model: ModelSupplierType,
    side_cull_fn: SideCullFunctionType,
}

impl Block {
    pub fn new(identifier: Identifier, settings: BlockSettings) -> Self {
        let hardness = settings.hardness.unwrap_or(0.0);
        let hardness_5 = hardness * 5.0;
        let resistance = match settings.resistance {
            Some(res) => 3.0 * res,
            None => hardness_5,
        };
        // For minecraft b1.7.3 functional parity, but seems to never really be used?
        let resistance = resistance.max(hardness_5);

        let slipperiness = settings.slipperiness.unwrap_or(0.0);

        let transparent = settings.transparent.unwrap_or(false);


        let model_supplier = settings.model_supplier.unwrap_or(|x| { VoxelModel::from_template(template_models::missing()).bake() });
        let side_cull_fn = settings.side_cull_fn.unwrap_or(|dir| { true });

        Self {
            identifier,
            hardness,
            resistance,
            slipperiness,
            transparent,
            model: model_supplier,
            side_cull_fn,
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

    pub fn get_model(&self, meta: u32) -> BakedModel {
        let f: ModelSupplierType = self.model;
        f(meta)
    }

    pub fn culls_side(&self, dir: Direction) -> bool {
        let f: SideCullFunctionType = self.side_cull_fn;
        f(dir)
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

pub struct BlockState {}

impl BlockState {}
