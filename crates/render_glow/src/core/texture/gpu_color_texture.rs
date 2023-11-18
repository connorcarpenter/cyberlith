
use crate::core::{GpuTexture2D, Program};

///
/// A reference to some type of texture containing colors.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub enum GpuColorTexture<'a> {
    /// A single 2D texture.
    Single(&'a GpuTexture2D),
}

impl GpuColorTexture<'_> {
    ///
    /// Returns the width of the color texture in texels.
    ///
    pub fn width(&self) -> u32 {
        match self {
            GpuColorTexture::Single(texture) => texture.width(),
        }
    }

    ///
    /// Returns the height of the color texture in texels.
    ///
    pub fn height(&self) -> u32 {
        match self {
            GpuColorTexture::Single(texture) => texture.height(),
        }
    }

    ///
    /// Returns the fragment shader source for using this texture in a shader.
    ///
    pub fn fragment_shader_source(&self) -> String {
        match self {
            Self::Single(_) => "
                uniform sampler2D colorMap;
                vec4 sample_color(vec2 uv)
                {
                    return texture(colorMap, uv);
                }"
            .to_owned(),
        }
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        match self {
            Self::Single(texture) => program.use_texture("colorMap", texture),
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
