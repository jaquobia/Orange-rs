/// A reference to some image in memory
///
#[derive(Copy, Clone, Debug)]
pub enum TextureObject {
    Texture2D {},
    TextureArray {},
    TextureAtlas {},
    AtlasTexture {},
    Texture3D {},
}