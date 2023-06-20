use glow::HasContext;

use render_api::base::*;

use crate::core::{Context, DepthTarget, texture::*};

///
/// A 2D depth texture that can be rendered into and read from. See also [RenderTarget] and [DepthTarget].
///
pub struct GpuDepthTexture2D {
    id: glow::Texture,
    width: u32,
    height: u32,
}

impl GpuDepthTexture2D {
    ///
    /// Constructs a new 2D depth texture.
    ///
    pub fn new<T: DepthTextureDataType>(
        width: u32,
        height: u32,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
    ) -> Self {
        let id = generate();
        let texture = Self { id, width, height };
        texture.bind();
        set_parameters(
            glow::TEXTURE_2D,
            Interpolation::Nearest,
            Interpolation::Nearest,
            wrap_s,
            wrap_t,
            None,
        );
        unsafe {
            Context::get().tex_storage_2d(
                glow::TEXTURE_2D,
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
    pub fn as_depth_target(&mut self) -> DepthTarget<'_> {
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
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::TEXTURE_2D,
                Some(self.id),
                0,
            );
        }
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_2D, Some(self.id));
        }
    }
}

impl Drop for GpuDepthTexture2D {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}
