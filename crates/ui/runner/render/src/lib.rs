use asset_loader::{AssetHandle, AssetManager};
use render_api::{components::RenderLayer, resources::RenderFrame};
use ui_runner::{UiManager, UiRuntime};

use crate::ui_renderer::UiRenderer;

mod ui_renderer;

pub trait UiRender {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiRuntime>,
    );
}

impl UiRender for UiManager {
    fn draw_ui(
        &self,
        asset_manager: &AssetManager,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiRuntime>,
    ) {
        UiRenderer::draw_ui(
            self,
            asset_manager,
            render_frame,
            render_layer_opt,
            &self.blinkiness,
            ui_handle,
        );
    }
}
