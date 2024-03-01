use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::system::ResMut;
use bevy_log::LogPlugin;

use naia_bevy_client::{ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin};

use asset_render::AssetPlugin;
use bevy_http_client::HttpClientPlugin;
use input::{Input, InputPlugin};
use filesystem::FileSystemPlugin;
use render_api::RenderApiPlugin;

use session_server_naia_proto::protocol as session_server_naia_protocol;
use world_server_naia_proto::{protocol as world_server_naia_protocol, components::{Alt1, Main, Position}};

use crate::{embedded_asset::handle_embedded_asset_event, world_events::InsertAssetRefEvent, connection_manager::{ConnectionManager, SessionConnectEvent}, asset_ref_processor::AssetRefProcessor, asset_cache::{AssetCache, AssetLoadedEvent}, client_markers::{Session, World}, InsertComponentEvent, renderer::RendererPlugin, world_events};

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
            .add_systems(Startup, engine_startup)
            // asset cache stuff, todo: maybe refactor out?
            .insert_resource(AssetCache::new("assets"))
            .add_event::<AssetLoadedEvent>()
            .add_systems(Update, AssetCache::handle_load_asset_tasks)
            .add_systems(Update, AssetCache::handle_save_asset_tasks)

            // connection manager stuff, maybe refactor out into a plugin?
            .init_resource::<ConnectionManager>()
            .add_event::<SessionConnectEvent>()
            .add_systems(Update, ConnectionManager::handle_connection)
            .add_systems(Update, ConnectionManager::handle_session_connect_events)
            .add_systems(Update, ConnectionManager::handle_session_message_events)
            .add_systems(Update, ConnectionManager::handle_session_request_events)
            .add_systems(Update, ConnectionManager::handle_world_connect_events)

            // asset ref processing stuff
            .init_resource::<AssetRefProcessor>()
            .add_systems(Update, AssetRefProcessor::handle_asset_loaded_events)

            // world component insert stuff
            .add_event::<InsertComponentEvent<Position>>()
            .add_event::<InsertAssetRefEvent<Main>>()
            .add_event::<InsertAssetRefEvent<Alt1>>()
            .add_systems(Startup, world_events::insert_component_event_startup)
            .add_systems(Update, world_events::insert_component_events)

            // embedded asset
            .add_systems(Update, handle_embedded_asset_event)
        ;
    }
}

fn engine_startup(mut input: ResMut<Input>) {
    input.set_enabled(true);
}