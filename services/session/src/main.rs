mod asset;
mod http;
mod region;
mod session_instance;
mod social;
mod user;
mod world;

use std::collections::HashMap;
use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;
use bevy_http_server::executor;
use config::{SESSION_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY};
use naia_bevy_server::{transport::webrtc, ServerConfig};

use crate::{
    asset::AssetPlugin,
    http::HttpPlugin,
    region::RegionPlugin,
    session_instance::SessionInstance,
    social::SocialPlugin,
    user::UserPlugin,
    world::{WorldPlugin, WorldServerMessage},
};

#[derive(Resource)]
struct WorldRoomManager {
    lobbies: HashMap<String, Lobby>,
    pending_requests: Vec<String>,
    room_keys: HashMap<String, String>, // LobbyId to RoomKey mapping
}

struct Lobby {
    id: String,
    user_ids: Vec<String>,
    ready: bool,
}

impl WorldRoomManager {
    fn new() -> Self {
        WorldRoomManager {
            lobbies: HashMap::new(),
            pending_requests: Vec::new(),
            room_keys: HashMap::new(),
        }
    }

    fn add_lobby(&mut self, lobby_id: String, user_ids: Vec<String>) {
        self.lobbies.insert(lobby_id.clone(), Lobby {
            id: lobby_id,
            user_ids,
            ready: true,
        });
    }

    fn get_lobby_ready_for_world_room(&mut self) -> Option<&Lobby> {
        self.lobbies.values().find(|lobby| lobby.ready && !self.pending_requests.contains(&lobby.id))
    }

    fn mark_lobby_as_pending(&mut self, lobby_id: &str) {
        self.pending_requests.push(lobby_id.to_string());
    }

    fn set_room_key(&mut self, lobby_id: &str, room_key: String) {
        self.room_keys.insert(lobby_id.to_string(), room_key);
    }

    fn get_room_key(&self, lobby_id: &str) -> Option<&String> {
        self.room_keys.get(lobby_id)
    }
}

#[derive(Resource)]
struct TokenManager {
    tokens: HashMap<String, String>, // UserId to Token mapping
}

impl TokenManager {
    fn new() -> Self {
        TokenManager {
            tokens: HashMap::new(),
        }
    }

    fn generate_token(&mut self, user_id: &str, lobby_id: &str, room_key: &str) -> String {
        let token = format!("{}:{}:{}", user_id, lobby_id, room_key);
        self.tokens.insert(user_id.to_string(), token.clone());
        token
    }
}

fn main() {
    logging::initialize();
    executor::setup(SESSION_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    let instance_secret = random::generate_random_string(16);
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(61);
    let world_connect_resend_rate = Duration::from_secs(5);

    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(HttpPlugin::new())
        .add_plugins(RegionPlugin::new(
            registration_resend_rate,
            region_server_disconnect_timeout,
        ))
        .add_plugins(SocialPlugin::new())
        .add_plugins(WorldPlugin::new(world_connect_resend_rate))
        .add_plugins(AssetPlugin::new())
        .add_plugins(UserPlugin::new())
        // Resources
        .insert_resource(SessionInstance::new(&instance_secret))
        .insert_resource(WorldRoomManager::new())
        .insert_resource(TokenManager::new())
        // Systems
        .add_systems(Update, request_world_room_creation)
        .add_systems(Update, handle_start_match_button_clicked)
        // Run App
        .run();
}

fn request_world_room_creation(
    mut world_room_manager: ResMut<WorldRoomManager>,
    mut world_server: ResMut<ServerConfig>,
) {
    if let Some(lobby) = world_room_manager.get_lobby_ready_for_world_room() {
        let message = WorldServerMessage::CreateWorldRoom {
            lobby_id: lobby.id.clone(),
            user_ids: lobby.user_ids.clone(),
        };
        // TODO: Replace with actual method to send message to World Server
        // world_server.send_message(message);
        world_room_manager.mark_lobby_as_pending(&lobby.id);
        logging::info!("Requested WorldRoom creation for lobby {}", lobby.id);
    }
}

fn handle_start_match_button_clicked(
    mut world_room_manager: ResMut<WorldRoomManager>,
    mut token_manager: ResMut<TokenManager>,
    mut world_server: ResMut<ServerConfig>,
    // Add other necessary resources or queries
) {
    // This is a placeholder for the actual event handling
    // In a real implementation, you'd listen for the StartMatchButtonClickedEvent

    if let Some(lobby) = world_room_manager.get_lobby_ready_for_world_room() {
        // Request WorldRoom creation
        let message = WorldServerMessage::CreateWorldRoom {
            lobby_id: lobby.id.clone(),
            user_ids: lobby.user_ids.clone(),
        };
        // TODO: Replace with actual method to send message to World Server
        // let room_key = world_server.send_message(message);
        let room_key = "placeholder_room_key".to_string(); // Replace with actual room key

        // Store the room key
        world_room_manager.set_room_key(&lobby.id, room_key.clone());

        // Generate and distribute tokens
        for user_id in &lobby.user_ids {
            let token = token_manager.generate_token(user_id, &lobby.id, &room_key);
            // TODO: Send token to user
            logging::info!("Generated token for user {} in lobby {}", user_id, lobby.id);
        }

        world_room_manager.mark_lobby_as_pending(&lobby.id);
        logging::info!("Started match for lobby {}", lobby.id);
    }
}