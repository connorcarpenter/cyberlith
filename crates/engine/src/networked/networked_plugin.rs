use std::sync::{Arc, RwLock};

use bevy_app::{App, Plugin, Startup, Update};

use kernel::http::CookieStore;
use naia_bevy_client::{ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin};

use session_server_naia_proto::protocol as session_server_naia_protocol;
use world_server_naia_proto::{
    components::{Alt1, Main, Position},
    protocol as world_server_naia_protocol,
};

use super::{
    asset_cache_checker::AssetCacheChecker,
    asset_ref_processor::AssetRefProcessor,
    client_markers::{Session, World},
    connection_manager::ConnectionManager,
    world_events,
    world_events::{InsertAssetRefEvent, InsertComponentEvent},
};
use crate::EnginePlugin;

pub struct NetworkedEnginePlugin {
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl NetworkedEnginePlugin {
    pub fn new(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self {
        Self { cookie_store_opt }
    }
}

impl Plugin for NetworkedEnginePlugin {
    fn build(&self, app: &mut App) {
        let engine_plugin = EnginePlugin::new(self.cookie_store_opt.clone());

        app.add_plugins(engine_plugin)
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
            .add_systems(Update, ConnectionManager::handle_connection)
            .add_systems(Update, ConnectionManager::handle_session_connect_events)
            .add_systems(Update, ConnectionManager::handle_session_disconnect_events)
            .add_systems(Update, ConnectionManager::handle_session_reject_events)
            .add_systems(Update, ConnectionManager::handle_session_message_events)
            .add_systems(Update, ConnectionManager::handle_session_request_events)
            .add_systems(Update, ConnectionManager::handle_world_connect_events)
            // asset ref processing stuff
            .init_resource::<AssetRefProcessor>()
            .add_systems(Startup, AssetRefProcessor::init_asset_loaded_events)
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
