mod asset_manager;
mod global;
mod http_server;
mod naia;
mod region_connection;
mod user_connection;

use std::time::Duration;
use std::collections::HashMap;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::{ResMut, Res};
use bevy_ecs::prelude::Resource;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
    Server, RoomKey, ConnectionEvent,
};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::{executor, HttpServerPlugin};
use config::{TOTAL_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY};
use world_server_http_proto::protocol as http_protocol;
use world_server_naia_proto::protocol as naia_protocol;

use crate::asset_manager::AssetManager;

struct WorldRoom {
    room_key: RoomKey,
    lobby_id: String,
    user_ids: Vec<String>,
}

#[derive(Resource)]
struct WorldRoomManager {
    rooms: HashMap<String, WorldRoom>, // LobbyId to WorldRoom mapping
}

impl WorldRoomManager {
    fn new() -> Self {
        WorldRoomManager {
            rooms: HashMap::new(),
        }
    }

    fn create_room(&mut self, lobby_id: String, user_ids: Vec<String>, room_key: RoomKey) {
        let world_room = WorldRoom {
            room_key,
            lobby_id: lobby_id.clone(),
            user_ids,
        };
        self.rooms.insert(lobby_id, world_room);
    }

    fn get_room(&self, lobby_id: &str) -> Option<&WorldRoom> {
        self.rooms.get(lobby_id)
    }
}

fn handle_world_room_creation(
    mut server: ResMut<Server>,
    mut world_room_manager: ResMut<WorldRoomManager>,
    mut events: EventReader<WorldServerMessage>,
) {
    for event in events.read() {
        if let WorldServerMessage::CreateWorldRoom { lobby_id, user_ids } = event {
            let room_key = server.create_room();
            world_room_manager.create_room(lobby_id.clone(), user_ids.clone(), room_key);
            // TODO: Send room_key back to Session Server
            logging::info!("Created WorldRoom for lobby {}", lobby_id);
        }
    }
}

fn handle_user_connection(
    mut server: ResMut<Server>,
    world_room_manager: Res<WorldRoomManager>,
) {
    while let Some(connection_event) = server.receive_connection_event() {
        if let ConnectionEvent::Connect(user_key) = connection_event {
            let token = server.user_data(&user_key).unwrap();
            let parts: Vec<&str> = token.split(':').collect();
            if parts.len() == 3 {
                let (user_id, lobby_id, _) = (parts[0], parts[1], parts[2]);
                if let Some(world_room) = world_room_manager.get_room(lobby_id) {
                    server.room_add_user(&world_room.room_key, &user_key);
                    logging::info!("User {} connected to WorldRoom for lobby {}", user_id, lobby_id);
                } else {
                    logging::error!("WorldRoom not found for lobby {}", lobby_id);
                }
            } else {
                logging::error!("Invalid token format");
            }
        }
    }
}

fn main() {
    logging::initialize();
    executor::setup(WORLD_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(NaiaServerPlugin::new(
            NaiaServerConfig::default(),
            naia_protocol(),
        ))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        .add_plugins(HttpClientPlugin)
        // Resources
        .insert_resource(AssetManager::new())
        .insert_resource(WorldRoomManager::new())
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, http_server::init)
        .add_systems(Startup, naia::tick_events_startup)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                naia::tick_events,
                user_connection::recv_login_request,
                region_connection::recv_heartbeat_request,
                region_connection::recv_register_instance_response,
                region_connection::send_register_instance_request,
                region_connection::process_region_server_disconnect,
                asset_manager::update,
                handle_world_room_creation,
                handle_user_connection,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}