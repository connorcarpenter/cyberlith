use crate::core::{GpuTexture2D, Program};

///
/// A reference to some type of texture containing colors.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub struct GpuColorTexture<'a> {
    /// A single 2D texture.
    inner: &'a GpuTexture2D,
}

impl<'a> GpuColorTexture<'a> {
    pub fn new(inner: &'a GpuTexture2D) -> Self {
        Self { inner }
    }

    ///
    /// Returns the width of the color texture in texels.
    ///
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    ///
    /// Returns the height of the color texture in texels.
    ///
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    ///
    /// Returns the fragment shader source for using this texture in a shader.
    ///
    pub fn fragment_shader_source(&self) -> String {
        "
            uniform sampler2D colorMap;
            vec4 sample_color(vec2 uv)
            {
                return texture(colorMap, uv);
            }"
        .to_owned()
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        program.use_texture("colorMap", self.inner)
    }

    ///
    /// The resolution of the underlying texture if there is any.
    ///
    pub fn resolution(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    pub fn bind_as_color_target(&self, channel: u32, mip_level: u32) {
        self.inner.bind_as_color_target(channel, mip_level);
    }
}
