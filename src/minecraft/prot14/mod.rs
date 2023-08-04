use rustc_hash::FxHashMap as HashMap;
use serde_derive::Deserialize;
use serde_json::Value;

use super::{registry::Registry, identifier::Identifier};

#[derive(Deserialize)]
struct JsonBlockMap {
    pub id: u32,
    pub block: String,
    pub post_processing: Option<Value>,
}

#[derive(Deserialize)]
struct JsonBlockMapping {
    pub map: Vec<JsonBlockMap>
}

/// Blocks to post-process:
/// Fence -> Beams (85)
/// Chest -> Rotation (54)
/// Grass -> Snow (2)
/// Noteblock? ()
/// Jukebox? ()
/// Portal -> Orientation (90)
/// Locked Chest -> Rotation (95)
pub fn generate_block_to_state_map(registry: &Registry) -> HashMap<u16, usize> {
    let blocks = registry.get_block_register();
    let mut map = HashMap::default();
    let mapping_data: JsonBlockMapping = serde_json::from_str(std::fs::read_to_string("block_id_map.json").unwrap_or("".to_string()).as_str()).expect("No valid block_id_map.json");
    let air = registry.get_blockstate_register().get_index_from_identifier(&"minecraft:sponge".into());
    mapping_data.map.into_iter().for_each(|block_map| {
        let full_id = block_map.id as u16;
        let blockstate_id = block_map.block;
        let split_identifier: Vec<&str> = blockstate_id.split("#").collect();
        
        let block = split_identifier.first().expect(format!("Invalid block identifier in {}", blockstate_id).as_str());

        let identifier = Identifier::from_str(block);
        let blockstate = match blocks.get_element_from_identifier(&identifier) {
            Some(block) => {
                let mut current_blockstate = block.get_default_state();
                
                if let Some(properties) = split_identifier.get(1) {
                    for property in properties.split(",") {
                        let (name, value) = property.split_once("=").unwrap();
                        current_blockstate = current_blockstate.with(name, value);
                    }
                }
                // current_blockstate.get_state_identifier()
                let state_id = registry.get_blockstate_register().get_index_from_identifier(current_blockstate.get_state_identifier());
                state_id
            },
            _ => {log::warn!("Improperly matched id to state: {} -> {}", identifier, full_id); air},
        };
        map.insert(full_id, blockstate);
    }); 
    map
}
