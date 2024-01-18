use bevy_app::{App, Plugin};
use bevy_log::LogPlugin;

use asset::AssetPlugin;
use http::HttpClientPlugin;
use input::InputPlugin;
use render_api::RenderApiPlugin;

use crate::renderer::RendererPlugin;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy Plugins
            .add_plugins(LogPlugin::default())
            // Add Render Plugins
            .add_plugins(RenderApiPlugin)
            .add_plugins(RendererPlugin)
            // Add misc crates Plugins
            .add_plugins(InputPlugin)
            .add_plugins(AssetPlugin)
            .add_plugins(HttpClientPlugin);
    }
}
