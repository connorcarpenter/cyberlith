use render_api::base::CubeMapSide;

use crate::core::{Texture2D, Texture2DArray, TextureCubeMap};

///
/// A reference to some type of texture containing colors.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub enum ColorTexture<'a> {
    /// A single 2D texture.
    Single(&'a Texture2D),
    /// An array of 2D textures and a set of indices into the array.
    Array {
        texture: &'a Texture2DArray,
        layers: &'a [u32],
    },
    /// A cube map texture and a set of [CubeMapSide]s indicating the sides to use.
    CubeMap {
        texture: &'a TextureCubeMap,
        sides: &'a [CubeMapSide],
    },
}

impl ColorTexture<'_> {
    ///
    /// Returns the width of the color texture in texels.
    ///
    pub fn width(&self) -> u32 {
        match self {
            ColorTexture::Single(texture) => texture.width(),
            ColorTexture::Array { texture, .. } => texture.width(),
            ColorTexture::CubeMap { texture, .. } => texture.width(),
        }
    }

    ///
    /// Returns the height of the color texture in texels.
    ///
    pub fn height(&self) -> u32 {
        match self {
            ColorTexture::Single(texture) => texture.height(),
            ColorTexture::Array { texture, .. } => texture.height(),
            ColorTexture::CubeMap { texture, .. } => texture.height(),
        }
    }
}
