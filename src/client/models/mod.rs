pub mod model;
pub mod model_builder;

use std::error::Error;

use model::BakedModel;
use rustc_hash::FxHashMap as HashMap;
use serde_json::Value;
use crate::minecraft::{filetypes::MCBlockstateType, identifier::Identifier};

use self::model::{VoxelModel, VoxelRotation};

use super::textures::TextureObject;

type BlockstateParseResult<T> = Result<T, Box<dyn Error>>;

pub fn state_contains_property(property: &String, conditions: &String, blockstate_identifier_string: &String) -> bool {
    for condition in conditions.split("|") {
        let propeqcondition = format!("{}={}", property, condition);
        if blockstate_identifier_string.contains(&propeqcondition) {
            return true;
        }
    } 
    false
}

fn extract_model_rotation(model: &Value) -> Option<(u8, f64)> {
    if let Value::Number(x) = &model["x"] {
        Some((0, x.as_f64()))
    } else if let Value::Number(y) = &model["y"] {
        Some((1, y.as_f64()))
    } else if let Value::Number(z) = &model["z"] {
        Some((2, z.as_f64()))
    } else {
        None
    }.map(|(axis, angle)| (axis, angle.unwrap_or(0.0)))
}

fn parse_model_value_to_identifier_and_rotation(variant_model: &Value) -> Result<(Identifier, Option<(u8, f64)>), &'static str> {
    variant_model["model"].as_str().map(|model_name| {
        let mut ident = Identifier::from(model_name);
        if !ident.get_name().starts_with("block/") {
            ident = Identifier::new(ident.get_namespace().clone(), format!("block/{}", ident.get_name()));
        }
        let varient_rotation = extract_model_rotation(variant_model);
        (ident, varient_rotation)
    }).ok_or("Model identifier is not a string!")
}

fn parse_variant_identifier_and_rotation(state_variants: &HashMap<String, Value>, blockstate_identifier_string: &String) -> Result<(Identifier, Option<(u8, f64)>), &'static str> {
    for (variant_properties, variant_model) in state_variants {
        let mut valid_variant = true;
        for variant_property in variant_properties.split(",") {
            valid_variant &= blockstate_identifier_string.contains(variant_property);
        }
        if valid_variant {
            return parse_model_value_to_identifier_and_rotation(variant_model);
        }
    }
    Err("No properties matched!")
}

fn generate_variant_blockstate_model(state_variants: &HashMap<String, Value>, blockstate_identifier_string: &String, voxel_models: &HashMap<Identifier, VoxelModel>, textures: &HashMap<Identifier, TextureObject>)
    -> BlockstateParseResult<BakedModel> {
    let (model_identifier, variant_rotation) = parse_variant_identifier_and_rotation(state_variants, blockstate_identifier_string)?;
    let model = voxel_models.get(&model_identifier).cloned().ok_or("Invalid model identifier")?;
    let rotation = variant_rotation.map(|(axis, angle)| { VoxelRotation::new(angle as f32, axis, [8., 8., 8.], false) });
    Ok(model.bake_with_rotate(rotation, &textures))
}

fn generate_multipart_blockstate_model(multiparts: &Vec<Value>, blockstate_identifier_string: &String, voxel_models: &HashMap<Identifier, VoxelModel>, textures: &HashMap<Identifier, TextureObject>) 
    -> BlockstateParseResult<BakedModel> {
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
        let (model_identifier, model_rotation) = parse_model_value_to_identifier_and_rotation(applied)?;
        let rotation = model_rotation.map(|(axis, angle)| { VoxelRotation::new(angle as f32, axis, [8., 8., 8.], false) });
        applied_models.push((model_identifier, rotation));
    }

    let mut model = VoxelModel::new().bake(textures);
    for applied_model in applied_models {
        let baked_model = voxel_models[&applied_model.0].clone().bake_with_rotate(applied_model.1, textures);
        model.combine(&baked_model);
    }
    Ok(model)
}
 

pub fn generate_blockstate_model(blockstate_file: &MCBlockstateType, blockstate_identifier_string: &String, voxel_models: &HashMap<Identifier, VoxelModel>, textures: &HashMap<Identifier, TextureObject>) -> BlockstateParseResult<BakedModel> {
    match &blockstate_file {
        MCBlockstateType::variants(state_variants) => {
            generate_variant_blockstate_model(state_variants, blockstate_identifier_string, voxel_models, textures)
        },
        MCBlockstateType::multipart(multiparts) => {
            generate_multipart_blockstate_model(multiparts, blockstate_identifier_string, voxel_models, textures)
        }
    }

}
