use std::{io::Write, path::{PathBuf}, fs};

use serde::{Serialize, Deserialize};
use serde_json::Value;

mod mc_constants;
use mc_constants::*;

/// A struct that represents the whole manifest
#[derive(Serialize, Deserialize)]
struct Manifest {
    versions: Vec<ManifestVersion>,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, Value>,
}

/// A struct that represents a version inside the manifest
#[derive(Serialize, Deserialize)]
struct ManifestVersion {
    id: String,
    r#type: String,
    url: String,
    time: String,
    sha1: String,
}

fn main() {
    check_assets();
}

/// Get the version manifest from mojang's servers
async fn get_manifest() -> Manifest {
    let mani: Manifest = surf::get(MANIFEST_URL).recv_json()
        .await.expect("Couldn't recieve/parse the manifest");
    return mani;
}

/// Get the jar version json the manifest's version url, and extract the download url; enforces the id and sha1 matches
async fn get_jar_from_version(manifest: &Manifest) -> String {
    let version: &ManifestVersion = match manifest.versions.iter().find(|a| a.id == VERSION_ID) {
        Some(mv) => mv,
        _ => &manifest.versions[600]
    };
    assert_eq!(version.sha1, VERSION_SHA1);

    let a: Value = surf::get(version.url.clone()).recv_json().await.expect("Couldn't recieve the version json");
    String::from(a["downloads"]["client"]["url"].as_str().expect("Couldn't get the jar url"))
}

/// Download a jar from a url as a byte array
async fn get_jar(uri: &String) -> Vec<u8> {
    surf::get(uri).recv_bytes().await.expect("Couldn't recieve the jar")
}


/// Returns false if all the necessary images in the provided resources directory exists, and true if any are missing
fn is_download_necessary(resources_dir: &PathBuf) -> bool {
    for asset in VEC_ASSETS2 {
        if !resources_dir.join(asset).exists() {
            return true;
        }
    }
    false
}

/// Handle checking and downloading images from the official jar
pub fn check_assets() -> bool {
    let resources_dir = PathBuf::from("./").join("legacy_resources");
    if is_download_necessary(&resources_dir) {
        println!("Downloading Assets...");
        download_minecraft_client(&resources_dir);
        println!("Assets Downloaded");
    }
    false
}

/// Downloads the minecraft client and extracts the resources
fn download_minecraft_client(dir: &PathBuf) {

    let dir = PathBuf::from(dir); // Copy the dir
    let manifest = pollster::block_on(get_manifest());

    let jar_url = pollster::block_on(get_jar_from_version(&manifest));
    let jar_data = pollster::block_on(get_jar(&jar_url));

    // Write into a temporary file for zip archive purposes
    let mut temp_jar_zip_handle = tempfile::tempfile().unwrap();
    temp_jar_zip_handle.write(jar_data.as_slice()).unwrap();
    let mut jar_zip = zip::ZipArchive::new(temp_jar_zip_handle).unwrap();


    fs::create_dir_all(&dir).unwrap();

    // Extract assets
    for index in 0..jar_zip.len() {
        let mut entry = jar_zip.by_index(index).unwrap();
        let name = entry.name();

        let outpath = match entry.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if name.ends_with('/') {
            // This is an empty folder for some reason, ignore it
        } else {
            let mut file_path = dir.clone();
            file_path.push(&outpath);
            let name = if name.contains('/') {
                 &name[name.rfind('/').unwrap()+1..]
            } else {
                name
            };
            if !VEC_ASSETS.contains(&name) {
                continue;
            }

            // Create the folders to put the files in
            if let Some(p) = file_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                };
            }

            // Copy the image data into a file
            let mut outfile = fs::File::create(&file_path).unwrap();
            std::io::copy(&mut entry, &mut outfile).unwrap();
        }

    }
}




