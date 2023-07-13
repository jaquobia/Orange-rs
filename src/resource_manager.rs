use std::{path::PathBuf, marker::PhantomData};
use rustc_hash::FxHashMap as HashMap;

pub enum ResourceType {
    Zip(PathBuf),
    Dir(PathBuf)
}

struct JsonFileFormat;
struct TomlFileFormat;
struct PngFileFormat;


struct FileFormatReader<T> { _data: PhantomData<T> }
trait FileFormatReaderTrait {
    fn load_file();
}

impl FileFormatReaderTrait for FileFormatReader<JsonFileFormat> {
    fn load_file() {
       log::warn!("Json Reading!"); 
    }
}

impl FileFormatReaderTrait for FileFormatReader<TomlFileFormat> {
    fn load_file() {
        log::warn!("Toml Reading!");
    }
}

impl FileFormatReaderTrait for FileFormatReader<PngFileFormat> {
    fn load_file() {
        log::warn!("Png Reading!");
    }
}

pub enum ResourceFileFormat {
    JSON,
    TOML,
    PNG,
}

pub struct ResourceManager {
    resource_sources: Vec<ResourceType>,
}

impl ResourceManager {
    pub fn reload_resources<T>(&self, buffer: &mut HashMap<String, T>, format: ResourceFileFormat, supported_extensions: Vec<String>) {
        match format {
            ResourceFileFormat::JSON => { FileFormatReader::<JsonFileFormat>::load_file(); },
            ResourceFileFormat::TOML => { FileFormatReader::<TomlFileFormat>::load_file(); },
            ResourceFileFormat::PNG => { FileFormatReader::<PngFileFormat>::load_file(); }
        }
    }

    /** Apply a function to all files in dir and subdirs   
      Will crash if depth is greater than number of allowed open files per program 
      */
    fn iter_files_recursive<F: FnMut(&std::fs::DirEntry)>(path: PathBuf, file_funct: &mut F) {
        if !path.is_dir() {
            log::error!("Not a dir: {}", path.display());
            return;
        }

        for f in std::fs::read_dir(path).unwrap() {
            let entry = f.unwrap();
            let entry_path = entry.path();
            if entry_path.is_dir() {
                Self::iter_files_recursive(entry_path, file_funct);
            } else {
                file_funct(&entry);
            }
        }
    }
}
