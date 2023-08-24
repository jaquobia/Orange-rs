pub mod block_factory;
pub mod properties;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use rustc_hash::FxHashMap as HashMap;

use crate::minecraft::identifier::Identifier;
use crate::direction::Direction;
use crate::minecraft::registry::{Registerable, Registry};

use self::block_factory::BlockSettings;
use self::properties::{PropertyValueType, PropertyDefinition};

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

    side_cull_fn: SideCullFunctionType,

    state_manager: RefCell<StateManager>,
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

        let side_cull_fn = settings.side_cull_fn.unwrap_or(|_| { true });

        Self {
            identifier,
            hardness,
            resistance,
            slipperiness,
            transparent,
            full_block,
            properties,
            side_cull_fn,
            state_manager: RefCell::new(StateManager::new()),
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

    pub fn culls_side(&self, dir: Direction) -> bool {
        let f: SideCullFunctionType = self.side_cull_fn;
        f(dir)
    }

    pub fn get_default_state(&self) -> Rc<BlockState> {
        self.state_manager.borrow().get_default()
    }

    pub fn map_states(block: Rc<Self>, registry: &mut Registry) -> Vec<Rc<BlockState>> {

        // let block = self;
        let properties = &block.properties;
        let base_identifier = block.identifier.to_string();
        let block_id = registry.get_block_register().get_index_from_identifier(block.get_identifier());
        let mut varients: Vec<BlockStatePropertyMap> = vec![HashMap::default()];
        let mut prev_varient_index = 1;
        let mut varient_indexs = vec![];

        for p_list in properties {
            let current_property_name = &p_list.0;
            let current_property_definition_id = &p_list.1;
            let current_property_def = registry.get_property_register().get_element_from_identifier(current_property_definition_id).unwrap();
            let current_property_values = current_property_def.get_names();

            varient_indexs.push(prev_varient_index);
            prev_varient_index *= current_property_values.len();

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

        let mut varient_references = vec![];

        let state_first_id = registry.get_blockstate_register().get_next_index();
        let mut blockstate_id = state_first_id;
        let state_varients = varients.into_iter().map(|varient| {
            let property_string = properties.iter().map(|property| {
                let prop = registry.get_property_register().get_element_from_identifier(&property.1).unwrap();
                let value_name = varient.get(&property.0).map(|u|prop.value_to_name(u.0).unwrap()).unwrap();
                format!("{}={}", property.0, value_name)
            }).collect::<Vec<_>>().join(",");
            let varient_name = Identifier::from_str(format!("{}#{}", base_identifier, property_string).as_str());

            varient_references.push(blockstate_id);
            blockstate_id += 1;
            (varient_name, varient)
        }).collect::<Vec<_>>();

        let states = state_varients.into_iter().map(|(id, properties)| Rc::new(BlockState::new(block.clone(), block_id, id, properties))).collect::<Vec<_>>();

        let weak_states = states.iter().map(|state| Rc::downgrade(state)).collect::<Vec<_>>();
        {
            let mut state_manager = block.state_manager.borrow_mut();
            state_manager.siblings = weak_states;
            state_manager.state_indicies = varient_indexs;
            state_manager.properties = properties.iter().map(|(name, def)| (name.clone(), registry.get_property_register().get_element_from_identifier(&def).unwrap().clone())).collect::<Vec<(String, Rc<PropertyDefinition>)>>();
        }
        
        states
    }

}

impl Registerable for Block {
    fn get_identifier(&self) -> &Identifier {
        return &self.identifier;
    }
}

pub type BlockStatePropertyMap = HashMap<String, (PropertyValueType, Identifier)>;

pub struct StateManager {
    siblings: Vec<Weak<BlockState>>,
    state_indicies: Vec<usize>,
    properties: Vec<(String, Rc<PropertyDefinition>)>,
    default_index: usize,
}

impl StateManager {
    fn new() -> Self {
        Self {
            siblings: vec![],
            state_indicies: vec![],
            properties: vec![],
            default_index: 0,
        }
    }

    pub fn with<S: AsRef<str>>(&self, old_properties: &BlockStatePropertyMap, name: S, value: S) -> Rc<BlockState> {
        self.inner_with(old_properties, name.as_ref(), value.as_ref())
    }

    fn inner_with(&self, old_properties: &BlockStatePropertyMap, name: &str, value: &str) -> Rc<BlockState> {
        let state_index = self.inner_get_state_index(old_properties, name, value);
        self.siblings[state_index].upgrade().unwrap()
    }

    fn inner_get_state_index(&self, old_properties: &BlockStatePropertyMap, name: &str, value: &str) -> usize {
        let mut state_index = 0;
        for (index, property) in self.properties.iter().enumerate() {
            let property_offset: usize = self.state_indicies.get(index).cloned().unwrap_or(0);
            if property.0 == name {
                let a: usize = match property.1.name_to_value(value) {
                    Ok(value) => value as usize,
                    Err(_) => { log::error!("No legal value \"{}\" for property \"{}\"", value, name); continue; }
                };
                state_index += a * property_offset;
                // Find the value of the new property...
            } else {
                state_index += old_properties.get(&property.0).unwrap().0 as usize * property_offset;
            }
        }
        state_index
    }

    pub fn set_default(&mut self, properties: BlockStatePropertyMap) {
        self.default_index = 0;
    }

    pub fn get_default(&self) -> Rc<BlockState> {
        self.siblings[self.default_index].upgrade().unwrap()
    }
}


#[derive(Clone)]
pub struct BlockState {
    block: Rc<Block>,
    block_id: usize,
    state_identifier: Identifier,
    property_map: BlockStatePropertyMap,
}

impl BlockState {

    pub fn new(block: Rc<Block>, block_id: usize, varient: Identifier, property_map: BlockStatePropertyMap) -> Self {
        
        Self {
            block,
            block_id,
            state_identifier: varient,
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

    pub fn get_block(&self) -> Rc<Block> {
        self.block.clone()
    }

    pub fn get_block_id(&self) -> usize {
        self.block_id
    }

    pub fn get_block_identifier(&self) -> &Identifier {
        &self.block.identifier
    }

    pub fn get_state_identifier(&self) -> &Identifier {
        &self.state_identifier
    }

    pub fn with<S: AsRef<str>>(&self, name: S, value: S) -> Rc<Self> {
        self.block.state_manager.borrow().with(&self.property_map, name, value)
    }
}

impl Registerable for BlockState {
    fn get_identifier(&self) -> &Identifier {
        &self.state_identifier
    }
}
