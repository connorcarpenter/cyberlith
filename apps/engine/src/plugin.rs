use bevy_app::{App, Plugin, Update};
use bevy_log::LogPlugin;

use naia_bevy_client::{ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin};

use asset_render::AssetPlugin;
use bevy_http_client::HttpClientPlugin;
use input::InputPlugin;
use filesystem::FileSystemPlugin;
use render_api::RenderApiPlugin;

use session_server_naia_proto::protocol as session_server_naia_protocol;
use world_server_naia_proto::protocol as world_server_naia_protocol;

use crate::{
    asset_cache::{AssetCache, AssetLoadedEvent},
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
            .add_plugins(FileSystemPlugin)
            .add_plugins(NaiaClientPlugin::<Session>::new(
                NaiaClientConfig::default(),
                session_server_naia_protocol(),
            ))
            .add_plugins(NaiaClientPlugin::<World>::new(
                NaiaClientConfig::default(),
                world_server_naia_protocol(),
            ))
            // asset cache stuff, todo: maybe refactor out?
            .insert_resource(AssetCache::new("assets"))
            .add_systems(Update, AssetCache::handle_load_asset_tasks)
            .add_systems(Update, AssetCache::handle_save_asset_tasks)
            .add_event::<AssetLoadedEvent>()
        ;
    }
}
