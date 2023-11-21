use gl::HasContext;

use render_api::base::*;

use crate::core::{texture::*, Context, DepthTarget, Program};

///
/// A 2D depth texture that can be rendered into and read from. See also [RenderTarget] and [DepthTarget].
///
pub struct GpuDepthTexture2D {
    id: gl::Texture,
    width: u32,
    height: u32,
}

impl GpuDepthTexture2D {
    ///
    /// Constructs a new 2D depth texture.
    ///
    pub fn new<T: DepthTextureDataType>(width: u32, height: u32) -> Self {
        let id = generate();
        let texture = Self { id, width, height };
        texture.bind();
        set_parameters(gl::TEXTURE_2D);
        unsafe {
            Context::get().tex_storage_2d(
                gl::TEXTURE_2D,
                1,
                T::internal_format(),
                width as i32,
                height as i32,
            );
        }
        texture
    }

    ///
    /// Returns a [DepthTarget] which can be used to clear, write to and read from this texture.
    /// Combine this together with a [ColorTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
    ///
    pub fn as_depth_target(&self) -> DepthTarget<'_> {
        DepthTarget::new_texture2d(self)
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::core) fn bind_as_depth_target(&self) {
        unsafe {
            Context::get().framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                Some(self.id),
                0,
            );
        }
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(gl::TEXTURE_2D, Some(self.id));
        }
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
        program.use_depth_texture("depthMap", self);
    }
}

impl From<&CpuTexture2D> for GpuDepthTexture2D {
    fn from(cpu_data: &CpuTexture2D) -> Self {
        Self::new::<f32>(cpu_data.width(), cpu_data.height())
    }
}

impl Drop for GpuDepthTexture2D {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}

//pub fn new(inner: &'a GpuDepthTexture2D) -> Self {
//         Self { inner }
//     }
//
//     ///
//     /// Returns the width of the depth texture in texels.
//     ///
//     pub fn width(&self) -> u32 {
//         self.inner.width()
//     }
//
//     ///
//     /// Returns the height of the depth texture in texels.
//     ///
//     pub fn height(&self) -> u32 {
//         self.inner.height()
//     }
//
//
//     ///
//     /// The resolution of the underlying texture if there is any.
//     ///
//     pub fn resolution(&self) -> (u32, u32) {
//         (self.inner.width(), self.inner.height())
//     }
//
//     pub fn bind_as_depth_target(&self) {
//         self.inner.bind_as_depth_target();
//     }
