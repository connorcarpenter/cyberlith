use render_api::{base::CubeMapSide, components::Viewport};

use crate::core::{ClearState, DepthTexture, DepthTexture2D, DepthTexture2DArray, DepthTextureCubeMap, RenderTarget};
use crate::renderer::RenderTargetExt;

///
/// Adds additional functionality to clear, read from and write to a texture.
/// Use the `as_depth_target` function directly on the texture structs (for example [DepthTexture2D]) to construct a depth target.
/// Combine this together with a [ColorTarget] with [RenderTarget::new] to be able to write to both a depth and color target at the same time.
/// A depth target purely adds functionality, so it can be created each time it is needed, the actual data is saved in the texture.
///
#[derive(Clone)]
pub struct DepthTarget<'a> {
    target: DepthTexture<'a>,
}

impl<'a> RenderTargetExt for DepthTarget<'a> {
    ///
    /// Writes whatever rendered in the `render` closure into this depth target.
    ///
    fn write(&self, render: impl FnOnce()) -> &Self {
        self.as_render_target().write(render);
        self
    }
}

impl<'a> DepthTarget<'a> {
    pub(in crate::core) fn new_texture2d(texture: &'a DepthTexture2D) -> Self {
        Self {
            target: DepthTexture::Single(texture),
        }
    }

    pub(in crate::core) fn new_texture_cube_map(
        texture: &'a DepthTextureCubeMap,
        side: CubeMapSide,
    ) -> Self {
        Self {
            target: DepthTexture::CubeMap { texture, side },
        }
    }

    pub(in crate::core) fn new_texture_2d_array(
        texture: &'a DepthTexture2DArray,
        layer: u32,
    ) -> Self {
        Self {
            target: DepthTexture::Array { texture, layer },
        }
    }

    ///
    /// Clears the depth of this depth target as defined by the given clear state.
    ///
    pub fn clear(&self, clear_state: ClearState) -> &Self {
        self.as_render_target().clear(ClearState {
            depth: clear_state.depth,
            ..ClearState::none()
        });
        self
    }

    ///
    /// Returns the depth values in this depth target.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read(&self) -> Vec<f32> {
        self.as_render_target().read_depth()
    }

    ///
    /// Copies the content of the depth texture
    /// to the part of this depth target specified by the [Viewport].
    ///
    pub fn copy_from(&self, depth_texture: DepthTexture, viewport: Viewport) -> &Self {
        self.as_render_target()
            .copy_from_depth(depth_texture, viewport);
        self
    }

    pub(super) fn as_render_target(&self) -> RenderTarget<'a> {
        RenderTarget::new_depth(self.clone())
    }

    ///
    /// Returns the width of the depth target in texels, which is simply the width of the underlying texture.
    ///
    pub fn width(&self) -> u32 {
        match &self.target {
            DepthTexture::Single(texture) => texture.width(),
            DepthTexture::Array { texture, .. } => texture.width(),
            DepthTexture::CubeMap { texture, .. } => texture.width(),
        }
    }

    ///
    /// Returns the height of the depth target in texels, which is simply the height of the underlying texture.
    ///
    pub fn height(&self) -> u32 {
        match &self.target {
            DepthTexture::Single(texture) => texture.height(),
            DepthTexture::Array { texture, .. } => texture.height(),
            DepthTexture::CubeMap { texture, .. } => texture.height(),
        }
    }

    pub(super) fn bind(&self) {
        match &self.target {
            DepthTexture::Single(texture) => {
                texture.bind_as_depth_target();
            }
            DepthTexture::Array { texture, layer } => {
                texture.bind_as_depth_target(*layer);
            }
            DepthTexture::CubeMap { texture, side } => {
                texture.bind_as_depth_target(*side);
            }
        }
    }
}
