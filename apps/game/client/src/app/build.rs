use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::{RenderApiPlugin, Window};

use crate::app::{GamePlugin, RendererPlugin};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Add Render Plugins
        .add_plugin(RenderApiPlugin)
        .add_plugin(RendererPlugin)
        // Add Game Plugin
        .add_plugin(GamePlugin);
    app
}
