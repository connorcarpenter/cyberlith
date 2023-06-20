use glow::HasContext;

use render_api::{base::CubeSide, components::Viewport};

use crate::core::{
    ClearState, Context, GpuColorTexture, GpuTexture2D, GpuTexture2DArray, GpuTextureCube, RenderTarget,
    TextureDataType, WriteMask,
};
use crate::renderer::RenderTargetExt;

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
    target: GpuColorTexture<'a>,
}

impl<'a> RenderTargetExt for ColorTarget<'a> {
    ///
    /// Returns the width of the color target in texels.
    /// If using the zero mip level of the underlying texture, then this is simply the width of that texture, otherwise it is the width of the given mip level.
    ///
    fn width(&self) -> u32 {
        match self.target {
            GpuColorTexture::Single(texture) => texture.width(),
            GpuColorTexture::Array { texture, .. } => texture.width(),
            GpuColorTexture::CubeMap { texture, .. } => texture.width(),
        }
    }

    ///
    /// Returns the height of the color target in texels.
    /// If using the zero mip level of the underlying texture, then this is simply the height of that texture, otherwise it is the height of the given mip level.
    ///
    fn height(&self) -> u32 {
        match self.target {
            GpuColorTexture::Single(texture) => texture.height(),
            GpuColorTexture::Array { texture, .. } => texture.height(),
            GpuColorTexture::CubeMap { texture, .. } => texture.height(),
        }
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
        ColorTarget {
            target: GpuColorTexture::Single(texture),
        }
    }

    pub(in crate::core) fn new_texture_cube_map(
        texture: &'a GpuTextureCube,
        sides: &'a [CubeSide],
    ) -> Self {
        ColorTarget {
            target: GpuColorTexture::CubeMap { texture, sides },
        }
    }

    pub(in crate::core) fn new_texture_2d_array(
        texture: &'a GpuTexture2DArray,
        layers: &'a [u32],
    ) -> Self {
        ColorTarget {
            target: GpuColorTexture::Array { texture, layers },
        }
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
        color_texture: GpuColorTexture,
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
        match self.target {
            GpuColorTexture::Single(texture) => unsafe {
                context.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
                texture.bind_as_color_target(0, 0);
            },
            GpuColorTexture::Array { texture, layers } => unsafe {
                context.draw_buffers(
                    &(0..layers.len())
                        .map(|i| glow::COLOR_ATTACHMENT0 + i as u32)
                        .collect::<Vec<u32>>(),
                );
                (0..layers.len()).for_each(|channel| {
                    texture.bind_as_color_target(layers[channel], channel as u32, 0);
                });
            },
            GpuColorTexture::CubeMap { texture, sides } => unsafe {
                context.draw_buffers(
                    &(0..sides.len())
                        .map(|i| glow::COLOR_ATTACHMENT0 + i as u32)
                        .collect::<Vec<u32>>(),
                );
                (0..sides.len()).for_each(|channel| {
                    texture.bind_as_color_target(sides[channel], channel as u32, 0);
                });
            },
        }
    }
}
