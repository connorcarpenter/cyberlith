use glow::HasContext;

use render_api::base::*;

use crate::core::{Context, DepthTarget, texture::*};

///
/// An array of 2D depth textures that can be rendered into and read from. See also [RenderTarget] and [DepthTarget].
///
pub struct GpuDepthTexture2DArray {
    id: glow::Texture,
    width: u32,
    height: u32,
    depth: u32,
}

impl GpuDepthTexture2DArray {
    ///
    /// Creates a new array of depth textures.
    ///
    pub fn new<T: DepthTextureDataType>(
        width: u32,
        height: u32,
        depth: u32,
        wrap_s: Wrapping,
        wrap_t: Wrapping,
    ) -> Self {
        let id = generate();
        let texture = Self {
            id,
            width,
            height,
            depth,
        };
        texture.bind();
        set_parameters(
            glow::TEXTURE_2D_ARRAY,
            Interpolation::Nearest,
            Interpolation::Nearest,
            wrap_s,
            wrap_t,
            None,
        );
        unsafe {
            Context::get().tex_storage_3d(
                glow::TEXTURE_2D_ARRAY,
                1,
                T::internal_format(),
                width as i32,
                height as i32,
                depth as i32,
            );
        }
        texture
    }

    ///
    /// Returns a [DepthTarget] which can be used to clear, write to and read from the given layer of this texture.
    /// Combine this together with a [ColorTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
    ///
    pub fn as_depth_target(&mut self, layer: u32) -> DepthTarget<'_> {
        DepthTarget::new_texture_2d_array(self, layer)
    }

    /// The width of this texture.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of this texture.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The number of layers.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub(in crate::core) fn bind_as_depth_target(&self, layer: u32) {
        unsafe {
            Context::get().framebuffer_texture_layer(
                glow::DRAW_FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                Some(self.id),
                0,
                layer as i32,
            );
        }
    }

    pub(in crate::core) fn bind(&self) {
        unsafe {
            Context::get().bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.id));
        }
    }
}

impl Drop for GpuDepthTexture2DArray {
    fn drop(&mut self) {
        unsafe {
            Context::get().delete_texture(self.id);
        }
    }
}