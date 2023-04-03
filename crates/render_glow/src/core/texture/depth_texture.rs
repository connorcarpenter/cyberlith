use render_api::base::CubeMapSide;

use crate::core::{DepthTexture2D, DepthTexture2DArray, DepthTextureCubeMap};

///
/// A reference to some type of texture containing depths.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub enum DepthTexture<'a> {
    /// A single 2D texture.
    Single(&'a DepthTexture2D),
    /// An array of 2D textures and an index into the array.
    Array {
        texture: &'a DepthTexture2DArray,
        layer: u32,
    },
    /// A cube map texture and a [CubeMapSide] indicating the side to use.
    CubeMap {
        texture: &'a DepthTextureCubeMap,
        side: CubeMapSide,
    },
}

impl DepthTexture<'_> {
    ///
    /// Returns the width of the depth texture in texels.
    ///
    pub fn width(&self) -> u32 {
        match self {
            DepthTexture::Single(texture) => texture.width(),
            DepthTexture::Array { texture, .. } => texture.width(),
            DepthTexture::CubeMap { texture, .. } => texture.width(),
        }
    }

    ///
    /// Returns the height of the depth texture in texels.
    ///
    pub fn height(&self) -> u32 {
        match self {
            DepthTexture::Single(texture) => texture.height(),
            DepthTexture::Array { texture, .. } => texture.height(),
            DepthTexture::CubeMap { texture, .. } => texture.height(),
        }
    }
}
