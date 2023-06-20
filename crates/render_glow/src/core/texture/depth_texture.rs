use render_api::base::CubeMapSide;

use crate::core::{DepthTexture2D, DepthTexture2DArray, DepthTextureCubeMap, Program};

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

    ///
    /// Returns the fragment shader source for using this texture in a shader.
    ///
    pub fn fragment_shader_source(&self) -> String {
        match self {
            Self::Single { .. } => "
                uniform sampler2D depthMap;
                float sample_depth(vec2 uv)
                {
                    return texture(depthMap, uv).x;
                }"
                .to_owned(),
            Self::Array { .. } => "
                uniform sampler2DArray depthMap;
                uniform int depthLayer;
                float sample_depth(vec2 uv)
                {
                    return texture(depthMap, vec3(uv, depthLayer)).x;
                }"
                .to_owned(),
            Self::CubeMap { .. } => {
                unimplemented!()
            }
        }
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        match self {
            Self::Single(texture) => program.use_depth_texture("depthMap", texture),
            Self::Array { texture, layer } => {
                program.use_uniform("depthLayer", *layer);
                program.use_depth_texture_array("depthMap", texture);
            }
            Self::CubeMap { .. } => unimplemented!(),
        }
    }

    ///
    /// The resolution of the underlying texture if there is any.
    ///
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Single(texture) => (texture.width(), texture.height()),
            Self::Array { texture, .. } => (texture.width(), texture.height()),
            Self::CubeMap { texture, .. } => (texture.width(), texture.height()),
        }
    }
}
