use glow::HasContext;

use render_api::base::*;

use crate::core::{Context, DepthTarget, texture::*};

///
/// A depth texture cube map that can be rendered into and read from. See also [RenderTarget] and [DepthTarget].
///
pub struct GpuDepthTextureCube {
    id: glow::Texture,
    width: u32,
    height: u32,
}

impl GpuDepthTextureCube {
    ///
    /// Creates a new depth texture cube map.
    ///
    pub fn new<T: DepthTextureDataType>(
        width: u32,
        height: u32,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
        wrap_r: Wrapping,
    ) -> Self {
        let id = generate();
        let texture = Self { id, width, height };
        texture.bind();
        set_parameters(
            glow::TEXTURE_CUBE_MAP,
            Interpolation::Nearest,
            Interpolation::Nearest,
            wrap_s,
            wrap_t,
            Some(wrap_r),
        );
        unsafe {
            Context::get().tex_storage_2d(
                glow::TEXTURE_CUBE_MAP,
                1,
                T::internal_format(),
                width as i32,
                height as i32,
            );
        }
        texture
    }

    ///
    /// Returns a [DepthTarget] which can be used to clear, write to and read from the given side of this texture.
    /// Combine this together with a [ColorTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
    ///
    pub fn as_depth_target(&mut self, side: CubeSide) -> DepthTarget<'_> {
        DepthTarget::new_texture_cube_map(self, side)
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::core) fn bind_as_depth_target(&self, side: CubeSide) {
        unsafe {
            Context::get().framebuffer_texture_2d(
                glow::DRAW_FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                side.to_const(),
                Some(self.id),
                0,
            );
        }
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_CUBE_MAP, Some(self.id));
        }
    }
}

impl Drop for GpuDepthTextureCube {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}