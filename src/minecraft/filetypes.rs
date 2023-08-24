use rustc_hash::FxHashMap as HashMap;
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;

use crate::{models::model::{VoxelFace, VoxelRotation, VoxelElement}, direction::Direction};

pub mod mcmeta {
    use rustc_hash::FxHashMap as HashMap;
    use serde_derive::{Serialize, Deserialize};

    use self::{pack::PackInformation, animation::Animation, atlas::SpriteAtlas, villager::Villager, properties::Propterties};


    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum MCMeta {
        Pack {
            pack: pack::PackInformation,
            language: Option<HashMap<String, pack::PackLanguage>>,
            filter: pack::PackFilter,
        },
        Animation {
            animation: animation::Animation
        },
        /// Custom format by orange for dealing with the terrain.png
        Atlas {
            atlas: atlas::SpriteAtlas
        },
        Villager {
            villager: villager::Villager
        },
        /// Use default values if missing
        Properties {
            texture: properties::Propterties
        }
    }

    impl MCMeta {
        pub fn as_pack(self) -> Option<PackInformation> {
            match self { Self::Pack { pack, .. } => Some(pack), _ => { None } }
        }
        pub fn as_animation(self) -> Option<Animation> {
            match self { Self::Animation { animation, .. } => Some(animation), _ => { None } }
        }
        pub fn as_atlas(self) -> Option<SpriteAtlas> {
            match self { Self::Atlas { atlas, ..} => Some(atlas), _ => { None } }
        }
        pub fn as_villager(self) -> Option<Villager> {
            match self { Self::Villager { villager } => Some(villager), _ => { None } }
        }
        pub fn as_properties(self) -> Option<Propterties> {
            match self { Self::Properties { texture } => Some(texture), _ => { None } }
        }
    }

    pub mod pack {
        use serde_derive::{Serialize, Deserialize};
        use serde_json::Value;

        #[derive(Serialize, Deserialize)]
        pub struct PackInformation {
            pack_format: u32,
            description: InformationDescription,
        }

        #[derive(Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum InformationDescription {
            String(String),
            Tag(Value),
        }

        #[derive(Serialize, Deserialize)]
        pub struct PackLanguage {
            name: String,
            region: String,
            bidirection: Option<bool>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct PackFilter {
            block: Vec<FilterPattern>
        }

        #[derive(Serialize, Deserialize)]
        pub struct FilterPattern {
            namespace: String,
            path: String,
        }

    }

    pub mod animation {
        use serde_derive::{Serialize, Deserialize};
        /// Needs to be gathered from an outside [Value]
        #[derive(Serialize, Deserialize)]
        pub struct Animation {
            interpolate: Option<bool>,
            width: Option<u32>,
            height: Option<u32>,
            frametime: Option<u32>,
            frames: Option<Vec<Frame>>,
        }

        #[derive(Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum Frame {
            Index(u32),
            IndexTime(u32, u32),
        }
    }

    pub mod atlas {
        use serde_derive::{Serialize, Deserialize};

        use crate::minecraft::identifier::Identifier;


        #[derive(Serialize, Deserialize)]
        pub enum SpriteAtlas {
            /**
             *  The atlas is a grid of regularly sized textures  
             *  Does not expect all cells to be filled
             */
            #[serde(rename="uniform")]
            Uniform {
                /** The number of textures horizontally */
                across: u32,
                /** The number of textures vertically 
                 *  If not provided, will be assumed the same as across
                 */
                down: Option<u32>,
                textures: Vec<SpriteAtlasUniform>,
            },
            #[serde(rename="nonuniform")]
            NonUniform {
                textures: Vec<SpriteAtlasNonUniform>,
            },
        }

        impl SpriteAtlas {
            pub fn new_uniform(width: u32, height: Option<u32>) -> Self {
                Self::Uniform { across: width, down: height, textures: Vec::new(), }
            }

            pub fn new_nonuniform() -> Self {
                Self::NonUniform { textures: Vec::new(), }
            }

            pub fn insert_uniform_texture(&mut self, identifier: Identifier, slot: u32) {
                if let Self::Uniform { textures, .. } = self {
                    textures.push(SpriteAtlasUniform { identifier: identifier.to_string(), cell: slot });
                }
            }

            pub fn insert_non_uniform_texture(&mut self, identifier: Identifier, uv: [f32; 4]) {
                if let Self::NonUniform { textures, .. } = self {
                    textures.push(SpriteAtlasNonUniform { identifier: identifier.to_string(), uv });
                }
            }

            pub fn get_uniform_textures(&self) -> Vec<SpriteAtlasUniform> {
                if let Self::Uniform { textures, .. } = self {
                    textures.to_vec()
                } else {
                    Vec::new()
                }
            }

            pub fn get_non_uniform_textures(&self) -> Vec<SpriteAtlasNonUniform> {
                if let Self::NonUniform { textures, .. } = self {
                    textures.to_vec()
                } else {
                    Vec::new()
                }
            }
        }

        #[derive(Serialize, Deserialize, Clone)]
        pub struct SpriteAtlasUniform {
            pub cell: u32,
            pub identifier: String,
        }

        #[derive(Serialize, Deserialize, Clone)]
        pub struct SpriteAtlasNonUniform {
            /** The uv min and max 
             *  [u_min, v_min, u_max, v_max]
             */
            pub uv: [f32; 4],
            pub identifier: String,
        }
    }

    pub mod villager {
        use serde_derive::{Serialize, Deserialize};

        #[derive(Serialize, Deserialize)]
        pub struct Villager {
            hat: Hat
        }
        #[derive(Serialize, Deserialize)]
        pub enum Hat {
            Full,
            Partial,
        }
    }

    pub mod properties {
        use serde_derive::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        pub struct Propterties {
            /// Texture is blurred when close to the player, default false
            blur: Option<bool>,
            /// Texture is streatched instead of tiled, default false
            clamp: Option<bool>,
            mipmaps: Vec<u32>,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MCAtlasConfig {
    pub sources: Vec<MCAtlasSource>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCAtlasSource {
    #[serde(rename="directory")]
    Directory {
        source: String,
        prefix: String,
    },
    #[serde(rename="single")]
    Single {
        resource: String,
        sprite: String,
    },
    #[serde(rename="filter")]
    Filter {
        namespace: String,
        path: String,
    },
    #[serde(rename="unstitch")]
    Unstitch {
        resource: String,
        divisor_x: f32,
        divisor_y: f32,
        regions: Vec<MCAtlasUnstitchRegion>
    }
}


#[derive(Serialize, Deserialize)]
pub struct MCAtlasUnstitchRegion {
    pub sprite: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MCModel {
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

impl MCModel {
    pub fn get_parent(&self) -> Option<String> {
        self.parent.clone()
    }

    pub fn get_ambient_occlusion(&self) -> bool {
        self.ambient_occlusion.unwrap_or(true)
    }

    pub fn textures(&self) -> HashMap<String, String> {
        match &self.textures {
            Some(textures) => textures.clone(),
            None => HashMap::default(),
        }
    }

    pub fn display(&self) -> HashMap<String, MCModelDisplay> {
        match &self.display {
            Some(display) => display.clone(),
            None => HashMap::default(),
        }
    }

    pub fn elements(&self) -> Vec<MCModelElement> {
        match &self.elements {
            Some(elements) => elements.to_vec(),
            None => vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MCModelElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<MCModelRotation>,
    pub shade: Option<bool>,
    pub faces: HashMap<String, MCModelFace>, 
}

impl MCModelElement {
    pub fn to_voxel_element(&self) -> VoxelElement {
        let mut voxel_element = VoxelElement::new( self.from, self.to );
        voxel_element.with_shade_nc(self.shade.unwrap_or(true));
        if let Some(rotation) = &self.rotation {
            voxel_element.with_rotation_nc(rotation.to_voxel_rotation());
        }         
        for (face_name, face_dir) in [("north", Direction::North), ("south", Direction::South), ("east", Direction::East), ("west", Direction::West), ("up", Direction::Up), ("down", Direction::Down)] {
            if let Some(face) = self.faces.get(&face_name.to_string()) {
                let voxel_face = face.to_voxel_face();
                voxel_element.with_face_nc(voxel_face,face_dir);
            }
        }
        voxel_element
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MCModelRotation {
    pub origin: [f32; 3],
    pub axis: String,
    pub angle: f32,
    pub rescale: Option<bool>,
}

impl MCModelRotation {
    pub fn to_voxel_rotation(&self) -> VoxelRotation {
        let axis = match self.axis.as_str() {
            "x" => { 0 },
            "y" => { 1 },
            "z" => { 2 },
            _ => { 0 },
        };
        VoxelRotation::new(self.angle, axis, self.origin, self.rescale.unwrap_or(false))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MCModelFace {
    pub uv: Option<[f32; 4]>,
    pub texture: String,
    pub cullface: Option<String>,
    pub rotation: Option<f32>,
    pub tintindex: Option<i32>,
}

impl MCModelFace {
    pub fn to_voxel_face(&self) -> VoxelFace {
        let mut voxel_face = VoxelFace::new(self.texture.clone());
        if let Some(uv) = self.uv {
            voxel_face.with_uv_nc((uv[0], (uv[1])), (uv[2], uv[3]));
        }
        if let Some(rotation) = self.rotation {
            voxel_face.with_rotation_nc(rotation);
        }
        if let Some(cullface) = &self.cullface {
            if let Ok(cullface) = Direction::try_from(cullface.as_str()) {
                voxel_face.with_cullface_nc(cullface);
            }
        }
        if let Some(tint) = self.tintindex {
            voxel_face.with_tint_nc(tint);
        }
        voxel_face
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MCModelDisplay {
    pub rotation: [f32; 3],
    pub translation: [f32; 3],
    pub scale: [f32; 3],
}

#[derive(Serialize, Deserialize)]
pub enum MCBlockstateType {
    #[serde(rename="variants")]
    Variants(HashMap<String, Value>),
    #[serde(rename="multipart")]
    Multipart(Vec<Value>),
}
