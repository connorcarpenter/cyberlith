use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::{RenderApiPlugin, Window};

use render_glow::RenderGlowPlugin;

use crate::app::GamePlugin;

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Insert Window Resource
        // TODO: find out how to get window height & width
        .insert_resource(Window::new(1280, 720))
        // Add Render Plugins
        .add_plugin(RenderApiPlugin)
        .add_plugin(RenderGlowPlugin)
        // Add Game Plugin
        .add_plugin(GamePlugin);
    app
}
