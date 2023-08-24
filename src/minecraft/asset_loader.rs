use std::{path::PathBuf, io::{Cursor, Read}, ffi::OsStr};

use image::DynamicImage;
use rustc_hash::FxHashMap as HashMap;

use crate::resource_loader::{ResourceSystem, ResourceCategory};

use super::{filetypes::{MCModel, MCBlockstateType, mcmeta::{MCMeta, atlas::SpriteAtlas}, MCAtlasConfig}, identifier::Identifier};

type AssetResult = Result<(), Box<dyn std::error::Error>>;

fn create_missing_tex() -> DynamicImage {
    let mut rgb_tex = image::Rgb32FImage::new(2, 2);
    let pink_pixel = image::Rgb::<f32>([0.9725, 0.0, 0.9725]);
    let black_pixel = image::Rgb::<f32>([0.0, 0.0, 0.0]);
    rgb_tex.put_pixel(0, 0, pink_pixel);
    rgb_tex.put_pixel(0, 1, black_pixel);
    rgb_tex.put_pixel(1, 0, black_pixel);
    rgb_tex.put_pixel(1, 1, pink_pixel);
    DynamicImage::ImageRgb32F(rgb_tex)
}

pub struct AssetLoader {
    domain: String,
    categories: Vec<ResourceCategory>,

    model_files: HashMap<Identifier, MCModel>,
    blockstate_files: HashMap<Identifier, MCBlockstateType>,
    sprites: HashMap<Identifier, DynamicImage>,
    mcmeta: HashMap<Identifier, MCMeta>,
    shaders: HashMap<Identifier, String>,
    atlases: HashMap<Identifier, MCAtlasConfig>
}

impl AssetLoader {
    pub fn new() -> Self {
        let mut sprites = HashMap::default();
        sprites.insert(Identifier::new("minecraft", "block/missing"), create_missing_tex());
        Self {
            domain: String::from("assets"),
            categories: vec![
                ResourceCategory::new("textures", vec!["png", "mcmeta"]),
                ResourceCategory::new("models", vec!["json"]),
                ResourceCategory::new("blockstates", vec!["json"]),
                ResourceCategory::new("lang", vec!["json"]),
                ResourceCategory::new("shaders", vec!["wgsl"]),
                ResourceCategory::new("atlases", vec!["json"]),
            ],
            model_files: HashMap::default(),
            blockstate_files: HashMap::default(),
            sprites,
            mcmeta: HashMap::default(),
            shaders: HashMap::default(),
            atlases: HashMap::default(),
        }
    }

    pub fn preload<S: AsRef<str>>(&mut self, preload_file_name: S, assets_directory: &PathBuf) {
        if let Err(e) = self.inner_preload(preload_file_name.as_ref(), assets_directory) {
            log::error!("Something went wrong while preloading {}: {}", preload_file_name.as_ref(), e);
        }
    }

    pub fn models(&self) -> &HashMap<Identifier, MCModel> {
        &self.model_files
    }
    
    pub fn blockstates(&self) -> &HashMap<Identifier, MCBlockstateType> {
        &self.blockstate_files
    }

    pub fn sprites(&self) -> &HashMap<Identifier, DynamicImage> {
        &self.sprites
    }

    pub fn mcmeta(&self) -> &HashMap<Identifier, MCMeta> {
        &self.mcmeta
    }

    pub fn shaders(&self) -> &HashMap<Identifier, String> {
        &self.shaders
    }

    pub fn atlases(&self) -> &HashMap<Identifier, MCAtlasConfig> {
        &self.atlases
    }

    fn try_load_model(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(model) = serde_json::from_slice(contents) {
            self.model_files.insert(Identifier::new(namespace, file_name), model);
        }
    }

    fn try_load_blockstate(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(blockstate) = serde_json::from_slice(contents) {
            self.blockstate_files.insert(Identifier::new(namespace, file_name), blockstate);
        }
    }

    fn try_load_mcmeta(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(mcmeta) = serde_json::from_slice(contents) {
            self.mcmeta.insert(Identifier::new(namespace, file_name), mcmeta);
        }
    }

    fn try_load_sprite(&mut self, namespace: &str, file_name: &str, contents: &[u8]) -> AssetResult {
        use image::io::Reader as ImageReader;
        let image = ImageReader::new(Cursor::new(contents)).with_guessed_format()?.decode()?;
        self.sprites.insert(Identifier::new(namespace, file_name), image);
        Ok(())
    }

    fn try_load_shaders(&mut self, namespace: &str, file_name: &str, contents: &[u8]) -> AssetResult {
        let shader_string = String::from_utf8(contents.to_vec())?;
        self.shaders.insert(Identifier::new(namespace, file_name), shader_string);
        Ok(())
    }

    fn try_load_lang(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {

    }

    fn try_load_atlas(&mut self, namespace: &str, file_name: &str, contents: &[u8]) {
        if let Ok(atlas) = serde_json::from_slice(contents) {
            self.atlases.insert(Identifier::new(namespace, file_name), atlas);
        }
    }

    fn inner_preload(&mut self, preload_file_name: &str, assets_directory: &PathBuf) -> AssetResult {
        // let json_file_name = format!("{}.json", preload_file_name);
        // let json_file = std::fs::read_to_string(assets_directory.join(json_file_name))?;
        // let json = serde_json::from_str::<Value>(json_file.as_str())?;

        let jar_file_name = format!("{}.jar", preload_file_name);
        let jar_file_path = assets_directory.join(jar_file_name);
        log::warn!("Preloading jar file {}", jar_file_path.display());
        let jar_file = std::fs::read(jar_file_path)?;
        let mut zip = zip::ZipArchive::new(Cursor::new(jar_file))?;
        
        let mut buffer = vec![];
        for i in 0..zip.len() {
            buffer.clear();
            let mut file = zip.by_index(i)?;
            if file.is_dir() { continue; }
            let (name, extension, enclosed_name): (String, Option<String>, PathBuf) = {
                let os_str_to_str = |s: &OsStr| { s.to_string_lossy().to_string() };
                let enclosed_name = file.enclosed_name().ok_or("Invalid enclosed name for zip entry")?.to_owned();
                let name = enclosed_name.file_name().map(os_str_to_str).ok_or("Stem could not be acquired")?;
                let extension = enclosed_name.extension().map(os_str_to_str);
                
                (name, extension, enclosed_name)
            };
            // We dont care about the java code
            let extension = match extension {
                Some(extension) => {
                    if extension.eq_ignore_ascii_case("class") { continue; }
                    extension
                },
                None => { continue; }
            };
            // log::warn!("Preloading {}", enclosed_name.display());
            if extension.eq_ignore_ascii_case("png") {
                let bytes_read = file.read_to_end(&mut buffer)?;
                self.try_load_sprite("minecraft", &enclosed_name.to_string_lossy(), &buffer)?;
            } else if extension.eq_ignore_ascii_case("txt") {
                let bytes_read = file.read_to_end(&mut buffer)?;
            } else if extension.eq_ignore_ascii_case("lang") {

            }
        }

        Ok(())
    }
}

impl ResourceSystem for AssetLoader {
    fn domain(&self) -> &str {
        &self.domain
    }
    fn categories(&self) -> &[crate::resource_loader::ResourceCategory] {
        &self.categories
    }
    fn try_load_file(&mut self, category: &str, namespace: &str, file_name: &str, file_extension: &str, contents: &[u8]) {
        // log::warn!("loading {}:{}.{} from {} ({file_name})", namespace, file_name, file_extension, category);
        match category {
            "models" => { self.try_load_model(namespace, file_name, contents); },
            "textures" => {
                match file_extension {
                    "mcmeta" => { self.try_load_mcmeta(namespace, file_name, contents); },
                    #[allow(unused_must_use)] // Stop rust from annoying me about abusing results
                                              // for early return
                    "png" => { self.try_load_sprite(namespace, file_name, contents); },
                    _ => {},
                }
            },
            "blockstates" => { self.try_load_blockstate(namespace, file_name, contents); },
            "shaders" => { self.try_load_shaders(namespace, file_name, contents); }
            "lang" => { self.try_load_lang(namespace, file_name, contents); },
            "atlases" => { self.try_load_atlas(namespace, file_name, contents); },
            _ => {},
        }
    }
}
