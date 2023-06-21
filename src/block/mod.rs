pub mod block_factory;
pub mod properties;

use rustc_hash::FxHashMap as HashMap;

use crate::minecraft::identifier::Identifier;
use crate::client::models::model::{BakedModel, VoxelModel};
use crate::direction::Direction;
use crate::minecraft::registry::{Registerable, Registry};
use crate::minecraft::template_models;

use self::block_factory::BlockSettings;
use self::properties::PropertyValueType;

pub type ModelSupplierType = fn(u32) -> BakedModel;
pub type SideCullFunctionType = fn(Direction) -> bool;

/// Describes the properties of block in the world, how they look, how they interact with
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
    /// Full Block, determines if this block consists of the entire voxel region 1^3, used in AO (client)
    full_block: bool,

    /// A list of properties stored per block
    properties: Vec<(String, Identifier)>,

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

        let full_block = settings.full_block.unwrap_or(true);

        let properties = settings.properties.unwrap_or(Vec::default());

        let model_supplier = settings.model_supplier.unwrap_or(|_| { VoxelModel::from_template(template_models::missing()).bake() });
        let side_cull_fn = settings.side_cull_fn.unwrap_or(|_| { true });

        Self {
            identifier,
            hardness,
            resistance,
            slipperiness,
            transparent,
            full_block,
            properties,
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

    pub fn is_full_block(&self) -> bool { self.full_block }

    pub fn is_solid_block(&self) -> bool { self.is_full_block() && !self.is_transparent() }

    pub fn get_model(&self, meta: u32) -> BakedModel {
        let f: ModelSupplierType = self.model;
        f(meta)
    }

    pub fn culls_side(&self, dir: Direction) -> bool {
        let f: SideCullFunctionType = self.side_cull_fn;
        f(dir)
    }

    pub fn map_states(&self, registry: &mut Registry) {
        // Verify the properties and map to a list of the variant identifiers
        let all_values: Vec<(String, Identifier, &[String])> = self.properties.iter().map(|(property_name, property_definition_id)| {
            match registry.get_property_register().get_element_from_identifier(property_definition_id.clone()) {
                Some(property) => {
                   (property_name.clone(), property_definition_id.clone(), property.get_names())
                },
                None => { panic!("Unregistered Property: {}", property_definition_id); }
            }
        }).collect();
        
        let base_identifier = self.identifier.to_string();
        let mut varients: Vec<BlockStatePropertyMap> = vec![HashMap::default()];

        for p_list in all_values {
            let current_property_name = &p_list.0;
            let current_property_definition_id = &p_list.1;
            let current_property_values = p_list.2;
            let current_property_def = registry.get_property_register().get_element_from_identifier(current_property_definition_id.clone()).unwrap();
            let mut new_varients = vec![];
            for value in current_property_values {
                let modified_varients = varients.clone().into_iter().map(|mut state_map| { 
                    state_map.insert(current_property_name.clone(), (current_property_def.name_to_value(value).unwrap(), current_property_definition_id.clone()) ); 
                    state_map 
                } );
                new_varients.extend(modified_varients);
            }
            varients = new_varients;
        } // end

        for varient in varients {
            let property_string = self.properties.iter().map(|property| {
                                                             let prop = registry.get_property_register().get_element_from_identifier(property.1.clone()).unwrap();
                                                             let value_name = varient.get(&property.0).map(|u|prop.value_to_name(u.0).unwrap()).unwrap();
                                                             format!("{}={}", property.0, value_name)
            }).collect::<Vec<_>>().join(",");
            let varient_name = Identifier::from_str(format!("{}#{}", base_identifier, property_string).as_str());

            registry.get_blockstate_register_mut().insert(BlockState::new(self.identifier.clone(), varient_name, varient));
        }
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

pub type BlockStatePropertyMap = HashMap<String, (PropertyValueType, Identifier)>;

#[derive(Clone)]
pub struct BlockState {
    block: Identifier,
    varient: Identifier,
    property_map: BlockStatePropertyMap,
}

impl BlockState {

    pub fn new(block: Identifier, varient: Identifier, property_map: BlockStatePropertyMap) -> Self {
        
        Self {
            block,
            varient,
            property_map,
        }
    }

    pub fn get_property(&self, property_name: String) -> PropertyValueType {
        if let Some(p) = self.property_map.get(&property_name) {
            p.0
        } else {
            panic!("Tried to get invalid property: {}", property_name);
        }
    }

    pub fn get_properties(&self) -> &BlockStatePropertyMap {
        &self.property_map
    }

    pub fn get_block_identifier(&self) -> &Identifier {
        &self.block
    }

    pub fn get_property_identifier(&self) -> &Identifier {
        &self.varient
    }
}

impl Registerable for BlockState {
    fn get_identifier(&self) -> &Identifier {
        &self.varient
    }
}
