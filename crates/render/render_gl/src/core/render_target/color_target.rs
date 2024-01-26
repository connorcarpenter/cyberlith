use gl::HasContext;

use render_api::components::Viewport;

use crate::{
    core::{ClearState, Context, GpuTexture2D, RenderTarget, TextureDataType, WriteMask},
    renderer::RenderTargetExt,
};

///
/// Adds additional functionality to clear, read from and write to a texture.
/// Use the `as_color_target` function directly on the texture structs (for example [GpuTexture2D]) to construct a color target.
/// Combine this together with a [DepthTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
/// A color target purely adds functionality, so it can be created each time it is needed, the actual data is saved in the texture.
///
/// **Note:** [DepthTest] is disabled if not also writing to a [DepthTarget].
///
#[derive(Clone)]
pub struct ColorTarget<'a> {
    target: &'a GpuTexture2D,
}

impl<'a> RenderTargetExt for ColorTarget<'a> {
    ///
    /// Returns the width of the color target in texels.
    ///
    fn width(&self) -> u32 {
        self.target.width()
    }

    ///
    /// Returns the height of the color target in texels.
    ///
    fn height(&self) -> u32 {
        self.target.height()
    }

    ///
    /// Writes whatever rendered in the `render` closure into this color target.
    ///
    fn write(&self, render: impl FnOnce()) -> &Self {
        self.as_render_target().write(render);
        self
    }
}

impl<'a> ColorTarget<'a> {
    pub(in crate::core) fn new_texture2d(texture: &'a GpuTexture2D) -> Self {
        ColorTarget { target: texture }
    }

    ///
    /// Clears the color of this color target as defined by the given clear state.
    ///
    pub fn clear(&self, clear_state: ClearState) -> &Self {
        self.as_render_target().clear(ClearState {
            depth: None,
            ..clear_state
        });
        self
    }

    ///
    /// Returns the colors of the pixels in this color target.
    /// The number of channels per pixel and the data format for each channel is specified by the generic parameter.
    ///
    /// **Note:** On web, the data format needs to match the data format of the color texture.
    ///
    pub fn read<T: TextureDataType>(&self) -> Vec<T> {
        self.as_render_target().read_color()
    }

    ///
    /// Copies the content of the color texture as limited by the [WriteMask]
    /// to the part of this color target specified by the [Viewport].
    ///
    pub fn copy_from(
        &self,
        color_texture: GpuTexture2D,
        viewport: Viewport,
        write_mask: WriteMask,
    ) -> &Self {
        self.as_render_target()
            .copy_from_color(color_texture, viewport, write_mask);
        self
    }

    pub(super) fn as_render_target(&self) -> RenderTarget<'a> {
        RenderTarget::new_color(self.clone())
    }

    pub(super) fn bind(&self) {
        let context = Context::get();
        unsafe {
            context.draw_buffers(&[gl::COLOR_ATTACHMENT0]);
            self.target.bind_as_color_target(0, 0);
        }
    }
}
