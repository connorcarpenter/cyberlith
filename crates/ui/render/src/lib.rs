use asset_loader::{AssetHandle, AssetManager, UiConfigData};
use render_api::{resources::RenderFrame, components::RenderLayer};
use ui_loader::UiManager;

use crate::ui_renderer::UiRenderer;

mod ui_renderer;

pub trait UiRender {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiConfigData>,
    );
}

impl UiRender for UiManager {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiConfigData>,
    ) {
        UiRenderer::draw_ui(self, asset_manager, render_frame, render_layer_opt, &self.blinkiness, ui_handle);
    }
}