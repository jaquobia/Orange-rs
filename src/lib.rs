pub mod block;
pub mod direction;
pub mod game_version;
pub mod world;
pub mod math_helper;
pub mod util;
pub mod server;
pub mod packets;
pub mod entities;
pub mod resource_loader;

#[cfg(feature = "client")]
pub mod client;
pub mod minecraft;

use std::path::PathBuf;
use winit::window::Icon;

lazy_static::lazy_static! {
    pub static ref MC_HOME : PathBuf = {
        PathBuf::from("./")
    };
}

pub fn get_app_icon(name: &str) -> Option<Icon> {
    use image::GenericImageView;
    let icon = image::open(name).unwrap_or_else(|_err| {
        println!("Failed to load {}", name);
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10))
    });
    let (icon_width, icon_height) = icon.dimensions();
    return Some(Icon::from_rgba(icon.into_bytes(), icon_width, icon_height).unwrap());
}

pub enum MCThread<T> {
    Shutdown,
    Work(T),
}


