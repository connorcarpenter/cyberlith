use bevy_ecs::system::{Res, ResMut};

use asset_loader::AssetManager;
use render_api::resources::RenderFrame;
use ui_runner::UiManager;

use crate::ui_renderer::UiRender;

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    asset_manager: Res<AssetManager>,
    ui_manager: Res<UiManager>,
) {
    ui_manager.draw_ui(&asset_manager, &mut render_frame);
}