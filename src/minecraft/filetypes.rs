use rustc_hash::FxHashMap as HashMap;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;

use super::identifier::Identifier;

#[derive(Serialize, Deserialize)]
pub struct MCAtlasTextureFile {
    pub atlas: AtlasTextureFileType,
}

impl MCAtlasTextureFile {
    pub fn new(atlas: AtlasTextureFileType) -> Self {
        Self { atlas, }
    }
}

#[derive(Serialize, Deserialize)]
pub enum AtlasTextureFileType {
    /**
     *  The atlas is a grid of regularly sized textures  
     *  Does not expect all cells to be filled
     */
    Uniform {
        /** The number of textures horizontally */
        across: u32,
        /** The number of textures vertically 
         *  If not provided, will be assumed the same as across
         */
        down: Option<u32>,
        textures: Vec<UniformAtlasTextureType>,
    },
    NonUniform {
       textures: Vec<NonUniformAtlasTextureType>,
    },
}

impl AtlasTextureFileType {
    pub fn new_uniform(width: u32, height: Option<u32>) -> Self {
        Self::Uniform { across: width, down: height, textures: Vec::new(), }
    }

    pub fn new_nonuniform() -> Self {
        Self::NonUniform { textures: Vec::new(), }
    }

    pub fn insert_uniform_texture(&mut self, identifier: Identifier, slot: u32) {
        if let Self::Uniform { textures, .. } = self {
            textures.push(UniformAtlasTextureType { identifier: identifier.to_string(), cell: slot });
        }
    }

    pub fn insert_non_uniform_texture(&mut self, identifier: Identifier, uv: [f32; 4]) {
        if let Self::NonUniform { textures, .. } = self {
            textures.push(NonUniformAtlasTextureType { identifier: identifier.to_string(), uv });
        }
    }

    pub fn get_uniform_textures(&self) -> Vec<UniformAtlasTextureType> {
        if let Self::Uniform { textures, .. } = self {
            textures.to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn get_non_uniform_textures(&self) -> Vec<NonUniformAtlasTextureType> {
        if let Self::NonUniform { textures, .. } = self {
            textures.to_vec()
        } else {
            Vec::new()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UniformAtlasTextureType {
    pub cell: u32,
    pub identifier: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NonUniformAtlasTextureType {
    /** The uv min and max 
     *  [u_min, v_min, u_max, v_max]
     */
    pub uv: [f32; 4],
    pub identifier: String,
}

#[derive(Serialize, Deserialize)]
pub struct MCModelFile {
    /** Identifier of a source model to load elements from, will be overriden if elements is also
     * defined
     */
    parent: Option<String>,
    /** Whether or not to use ambient occlusion (only works in the parent model file)
     */
    ambient_occlusion: Option<bool>,
    /** A mapping of the texture variables to resource locations or another texture variable
     */
    textures: Option<HashMap<String, String>>,
    /** Defines how the model is displayed in different contexts: thirdperson_righthand,
     * thirdperson_lefthand, firstperson_righthand, firstperson_lefthand, gui, head, ground, fixed
    */
    display: Option<HashMap<String, MCModelDisplay>>,
    /** The list of elements in the model, overrides a parent elements tag
     */
    elements: Option<Vec<MCModelElement>>,
}

#[derive(Serialize, Deserialize)]
pub struct MCModelElement {
    from: [f32; 3],
    to: [f32; 3],
    rotation: Option<MCModelRotation>,
    shade: Option<bool>,
    faces: HashMap<String, MCModelFace>, 
}

#[derive(Serialize, Deserialize)]
pub struct MCModelRotation {
    origin: [f32; 3],
    axis: String,
    angle: f32,
    rescale: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct MCModelFace {
    uv: Option<[f32; 4]>,
    texture: String,
    cullface: Option<String>,
    rotation: Option<f32>,
    tintindex: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct MCModelDisplay {
    rotation: [f32; 3],
    translation: [f32; 3],
    scale: [f32; 3],
}

#[derive(Serialize, Deserialize)]
pub enum MCBlockstateType {
    variants(HashMap<String, Value>),
    multipart(Vec<Value>),
}
