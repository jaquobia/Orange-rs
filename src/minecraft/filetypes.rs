use serde_derive::{Serialize, Deserialize};

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
