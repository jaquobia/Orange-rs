use std::{path::PathBuf, collections::HashSet};
use rustc_hash::FxHashSet;

#[derive(Clone, Debug)]
pub struct ResourceCategory {
    name: String,
    valid_extensions: FxHashSet<String>,
}

impl ResourceCategory {
    pub fn new(name: &'static str, extensions: Vec<&'static str>) -> Self {
        Self {
            name: name.to_string(),
            valid_extensions: HashSet::from_iter(extensions.into_iter().map(|s|s.to_string())),
        }
    }
}

pub trait ResourceSystem {
    fn domain(&self) -> &str;
    fn categories(&self) -> &[ResourceCategory];
    fn try_load_file(&mut self, category: &str, namespace: &str, file_name: &str, file_extension: &str, contents: &[u8]);
}

#[derive(Clone, Debug)]
pub enum ResourceSource {
    Zip(PathBuf),
    Folder(PathBuf)
}

#[derive(Clone, Debug)]
pub struct ResourceLoader {
    resource_sources: Vec<ResourceSource>,
}

impl ResourceLoader {
    
    pub fn new() -> Self {
        Self { resource_sources: vec![] }
    }

    pub fn set_sources(&mut self, sources: &[PathBuf]) {
        let sources: Vec<ResourceSource> = sources.iter().filter_map(|path| {
            if path.exists() {
                if path.is_dir() && !path.is_symlink() {
                    Some(ResourceSource::Folder(path.to_path_buf()))
                } else if path.is_file() {
                    Some(ResourceSource::Zip(path.to_path_buf()))
                } else {
                    log::warn!("Path {} is either an invalid folder or not a file", path.display());
                    None
                }
            } else {
                log::warn!("Path {} does not exist", path.display());
                None
            }
        }).rev().collect();
        self.resource_sources = sources;
    }

    pub fn reload_system<S: ResourceSystem>(&self, system: &mut S) {
        for source in &self.resource_sources {
            match source {
                ResourceSource::Folder(pack_path) => {
                    Self::reload_system_folder(pack_path.to_owned(), system);
                },
                ResourceSource::Zip(path) => {

                }
            }
        }
    }

    fn reload_system_folder<S: ResourceSystem>(pack_path: PathBuf, system: &mut S) {
        if !(pack_path.exists() && pack_path.is_dir()) {
            log::warn!("Path {} is an invald folder", pack_path.display());
            return;
        }
        let domain_path = pack_path.join(system.domain());
        if !(domain_path.exists() && domain_path.is_dir()) {
            log::warn!("Domain {} is not found in pack {}", domain_path.display(), pack_path.display());
            return;
        }

        let namespaces = match std::fs::read_dir(domain_path) {
            Ok(namespaces) => namespaces,
            Err(e) => {
                log::warn!("Couldnt walk the namespaces, {e}");
                return;
            }
        };
        for namespace in namespaces {
            let namespace = namespace.unwrap();
            let namespace_name = namespace.file_name().to_string_lossy().to_string();
            let namespace_path = namespace.path();
            for category in system.categories().to_vec() {
                let category_path = namespace_path.join(&category.name);
                if !category_path.exists() || category_path.is_symlink() || !category_path.is_dir() {
                    log::warn!("Expected category is missing or not a folder: {} in {}", category.name, category_path.display());
                    continue;
                }
                Self::iter_files_recursive(category_path.to_path_buf(), &mut |entry| {
                    let entry_path = entry.path();
                    let extension: String = entry_path.extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(String::new);
                    let dotextension = [".", extension.as_str()].join("");
                    let file_name = entry_path.strip_prefix(&category_path).unwrap().to_string_lossy().to_string().replace(&dotextension, "").replace("\\", "/");
                    // let file_name = entry.file_name().to_string_lossy().to_string();
                    if !category.valid_extensions.contains(&extension) {
                        return;
                    }
                    if let Ok(contents) = std::fs::read(entry_path) {
                        system.try_load_file(&category.name, &namespace_name, &file_name, &extension, &contents);
                    }
                });

            }
        }
    }

    /** Apply a function to all files in dir and subdirs   
      Will crash if depth is greater than number of allowed open files per program 
      */
    fn iter_files_recursive<F: FnMut(&std::fs::DirEntry)>(path: PathBuf, file_funct: &mut F) {
        if !path.is_dir() {
            log::warn!("Not a dir: {}", path.display());
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
