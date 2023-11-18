
use crate::core::{GpuDepthTexture2D, Program};

///
/// A reference to some type of texture containing depths.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub enum GpuDepthTexture<'a> {
    /// A single 2D texture.
    Single(&'a GpuDepthTexture2D),
}

impl GpuDepthTexture<'_> {
    ///
    /// Returns the width of the depth texture in texels.
    ///
    pub fn width(&self) -> u32 {
        match self {
            GpuDepthTexture::Single(texture) => texture.width(),
        }
    }

    ///
    /// Returns the height of the depth texture in texels.
    ///
    pub fn height(&self) -> u32 {
        match self {
            GpuDepthTexture::Single(texture) => texture.height(),
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
        }
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        match self {
            Self::Single(texture) => program.use_depth_texture("depthMap", texture),
        }
    }

    ///
    /// The resolution of the underlying texture if there is any.
    ///
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Single(texture) => (texture.width(), texture.height()),
        }
    }
}
