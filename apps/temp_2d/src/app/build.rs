use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::RenderApiPlugin;
use render_glow::RenderGlowPlugin;

use crate::app::{GamePlugin, RendererPlugin};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Render Plugins
        .add_plugin(RenderApiPlugin)
        .add_plugin(RenderGlowPlugin)
        // Add Game Plugin
        .add_plugin(GamePlugin);
    app
}
