use bevy_app::{App, Plugin};
use bevy_log::LogPlugin;

use asset_render::AssetPlugin;
use bevy_http_client::HttpClientPlugin;
use input::InputPlugin;
use naia_bevy_client::{ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin};
use render_api::RenderApiPlugin;

use session_server_naia_proto::protocol as session_server_naia_protocol;
use world_server_naia_proto::protocol as world_server_naia_protocol;

use crate::{
    client_markers::{Session, World},
    renderer::RendererPlugin,
};

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
            .add_plugins(HttpClientPlugin)
            .add_plugins(NaiaClientPlugin::<Session>::new(
                NaiaClientConfig::default(),
                session_server_naia_protocol(),
            ))
            .add_plugins(NaiaClientPlugin::<World>::new(
                NaiaClientConfig::default(),
                world_server_naia_protocol(),
            ));
    }
}
