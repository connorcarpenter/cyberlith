use asset_loader::{AssetHandle, AssetManager, UiData, UiManager};
use render_api::{resources::RenderFrame, components::RenderLayer};

use crate::ui_renderer::UiRenderer;

mod ui_renderer;

pub trait UiRender {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiData>,
    );
}

impl UiRender for UiManager {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiData>,
    ) {
        UiRenderer::draw_ui(self, asset_manager, render_frame, render_layer_opt, &self.blinkiness, ui_handle);
    }
}