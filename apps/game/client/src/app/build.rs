use bevy_app::App;
use bevy_log::LogPlugin;

use asset::AssetPlugin;
use input::InputPlugin;
use render_api::RenderApiPlugin;

use crate::app::{GamePlugin, RendererPlugin};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugins(LogPlugin::default())
        // Add Render Plugins
        .add_plugins(RenderApiPlugin)
        .add_plugins(RendererPlugin)
        // Add misc crates Plugins
        .add_plugins(InputPlugin)
        .add_plugins(AssetPlugin)
        // Add Game Plugin
        .add_plugins(GamePlugin);
    app
}
