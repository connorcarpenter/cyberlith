use bevy_app::App;
use bevy_log::LogPlugin;

use render_api::{RenderApiPlugin, Window};

use crate::app::{RendererPlugin, GamePlugin};

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
        .add_plugin(RendererPlugin)
        // Add Game Plugin
        .add_plugin(GamePlugin);
    app
}
