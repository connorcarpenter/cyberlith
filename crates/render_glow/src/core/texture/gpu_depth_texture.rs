use crate::core::{GpuDepthTexture2D, Program};

///
/// A reference to some type of texture containing depths.
///
#[derive(Clone, Copy)]
#[allow(missing_docs)]
pub struct GpuDepthTexture<'a> {
    inner: &'a GpuDepthTexture2D,
}

impl<'a> GpuDepthTexture<'a> {
    pub fn new(inner: &'a GpuDepthTexture2D) -> Self {
        Self { inner }
    }

    ///
    /// Returns the width of the depth texture in texels.
    ///
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    ///
    /// Returns the height of the depth texture in texels.
    ///
    pub fn height(&self) -> u32 {
        self.inner.height()
    }

    ///
    /// Returns the fragment shader source for using this texture in a shader.
    ///
    pub fn fragment_shader_source(&self) -> String {
        "
            uniform sampler2D depthMap;
            float sample_depth(vec2 uv)
            {
                return texture(depthMap, uv).x;
            }"
        .to_owned()
    }

    ///
    /// Sends the uniform data needed for this texture to the fragment shader.
    ///
    pub fn use_uniforms(&self, program: &Program) {
        program.use_depth_texture("depthMap", self.inner);
    }

    ///
    /// The resolution of the underlying texture if there is any.
    ///
    pub fn resolution(&self) -> (u32, u32) {
        (self.inner.width(), self.inner.height())
    }

    pub fn bind_as_depth_target(&self) {
        self.inner.bind_as_depth_target();
    }
}
