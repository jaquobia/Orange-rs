pub mod model;
pub mod model_builder;

use model::BakedModel;
use rustc_hash::FxHashMap as HashMap;
use serde_json::Value;
use crate::minecraft::{filetypes::MCBlockstateType, identifier::Identifier};

use self::model::{VoxelModel, VoxelRotation};

use super::textures::TextureObject;

pub fn state_contains_property(property: &String, conditions: &String, blockstate_identifier_string: &String) -> bool {
    for condition in conditions.split("|") {
        let propeqcondition = format!("{}={}", property, condition);
        if blockstate_identifier_string.contains(&propeqcondition) {
            return true;
        }
    } 
    false
}

pub fn generate_blockstate_model(blockstate_file: &MCBlockstateType, blockstate_identifier_string: &String, voxel_models: &HashMap<Identifier, VoxelModel>, textures: &HashMap<Identifier, TextureObject>) -> BakedModel {
    match &blockstate_file {
        MCBlockstateType::variants(state_variants) => {
            let mut model_identifier = Identifier::from_str(state_variants.get("").map(|v| v["model"].as_str().unwrap()).unwrap_or("minecraft:missing"));
            let mut variant_rotation = None;
            for (variant_properties, variant_model_name) in state_variants {
                let mut valid_variant = true;
                for variant_property in variant_properties.split(",") {
                    valid_variant &= blockstate_identifier_string.contains(variant_property);
                }
                if valid_variant {
                    model_identifier = Identifier::from(variant_model_name["model"].as_str().unwrap());
                    if variant_model_name["x"].is_number() {
                        variant_rotation = Some((0, variant_model_name["x"].as_f64()));
                    } else if variant_model_name["y"].is_number() {
                        variant_rotation = Some((1, variant_model_name["y"].as_f64()));
                    } else if variant_model_name["z"].is_number() {
                        variant_rotation = Some((2, variant_model_name["z"].as_f64()));
                    }
                }
            }

            if model_identifier.get_identifier_string() == "minecraft:missing" {
                log::warn!("Using missing model for {}", blockstate_identifier_string);
            }

            if !model_identifier.get_name().starts_with("block/") {
                model_identifier = Identifier::new(model_identifier.get_namespace().clone(), format!("block/{}", model_identifier.get_name()));
            }

            let model = match voxel_models.get(&model_identifier) {
                Some(model_file) => {
                    log::info!("Using model {} for blockstate {}", model_identifier, blockstate_identifier_string);
                    model_file
                },
                None => {
                    log::error!("Invalid model {} for blockstate {}!", model_identifier, blockstate_identifier_string);
                    voxel_models.get(&Identifier::from_str("minecraft:block/missing")).expect("No missing model!")
                }
            };
            let rotation = variant_rotation.map(|(axis, angle)| { VoxelRotation::new(angle.unwrap_or(0.) as f32, axis, [8., 8., 8.], false) });
            model.clone().bake_with_rotate(rotation, &textures)
        },
        MCBlockstateType::multipart(multiparts) => {
            log::warn!("Checking multipart model for {}", blockstate_identifier_string);
            let mut applied_models: Vec<(Identifier, Option<VoxelRotation>)> = vec![];
            for part in multiparts {
                let conditions_passed: bool = match &part["when"] {
                    Value::Object(values) => {
                        let mut passed = false;
                        for (property, conditions_list) in values {
                            if let Value::String(conditions) = conditions_list {
                                let property_passed = state_contains_property(property, conditions, blockstate_identifier_string);
                                passed |= property_passed;
                            }
                        }
                        passed
                    },
                    Value::Null => true,
                    _ => false,
                };
                if !conditions_passed { continue; }
                let applied = &part["apply"];
                let model_identifier = Identifier::from(applied["model"].as_str().unwrap());
                let model_rotation = if let Value::Number(x) = &applied["x"] {
                    Some((0, x))
                } else if let Value::Number(y) = &applied["y"] {
                    Some((1, y))
                } else if let Value::Number(z) = &applied["z"] {
                    Some((2, z))
                } else {
                    None
                };
                let rotation = model_rotation.map(|(axis, angle)| { VoxelRotation::new(angle.as_f64().unwrap_or(0.) as f32, axis, [8., 8., 8.], false) });
                applied_models.push((model_identifier, rotation));
            }

            let mut model = VoxelModel::new().bake(textures);

            log::warn!("Combining {} parts together!", applied_models.len());
            for applied_model in applied_models {
                let baked_model = voxel_models[&applied_model.0].clone().bake_with_rotate(applied_model.1, textures);
                model.combine(&baked_model);
            }
            model

        }
    }

}
