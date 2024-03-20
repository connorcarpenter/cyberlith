use bevy_app::{App, Plugin, Startup, Update};

use naia_bevy_client::{ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin};

use session_server_naia_proto::protocol as session_server_naia_protocol;
use world_server_naia_proto::{
    components::{Alt1, Main, Position},
    protocol as world_server_naia_protocol,
};

use crate::EnginePlugin;

use super::{
    asset_cache_checker::AssetCacheChecker,
    asset_ref_processor::AssetRefProcessor,
    client_markers::{Session, World},
    connection_manager::{ConnectionManager, SessionConnectEvent},
    world_events,
    world_events::{InsertAssetRefEvent, InsertComponentEvent},
};

pub struct NetworkedEnginePlugin;

impl Plugin for NetworkedEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnginePlugin)
            .add_plugins(NaiaClientPlugin::<Session>::new(
                NaiaClientConfig::default(),
                session_server_naia_protocol(),
            ))
            .add_plugins(NaiaClientPlugin::<World>::new(
                NaiaClientConfig::default(),
                world_server_naia_protocol(),
            ))
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
            .init_resource::<AssetCacheChecker>()
            .add_systems(Update, AssetCacheChecker::handle_load_asset_tasks)
            // world component insert stuff
            .add_event::<InsertComponentEvent<Position>>()
            .add_event::<InsertAssetRefEvent<Main>>()
            .add_event::<InsertAssetRefEvent<Alt1>>()
            .add_systems(Startup, world_events::insert_component_event_startup)
            .add_systems(Update, world_events::insert_component_events);
    }
}
