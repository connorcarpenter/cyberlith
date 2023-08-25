use bevy_app::App;
use bevy_log::LogPlugin;

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
        // Add Input Plugin
        .add_plugins(InputPlugin)
        // Add Game Plugin
        .add_plugins(GamePlugin);
    app
}
