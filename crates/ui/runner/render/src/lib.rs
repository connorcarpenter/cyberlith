use asset_loader::AssetManager;
use render_api::resources::RenderFrame;
use ui_runner::UiManager;

use crate::ui_renderer::UiRenderer;

mod ui_renderer;

pub trait UiRender {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
    );
}

impl UiRender for UiManager {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
    ) {
        let render_layer_opt = self.render_layer();
        if let Some(active_ui_handle) = self.active_ui() {
            UiRenderer::draw_ui(
                self,
                asset_manager,
                render_frame,
                render_layer_opt.as_ref(),
                &self.blinkiness,
                &active_ui_handle,
            );
        }
    }
}
