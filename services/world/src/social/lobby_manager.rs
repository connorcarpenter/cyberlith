use std::collections::HashMap;

use bevy_ecs::system::Resource;

use naia_bevy_server::{RoomKey};

use social_server_types::LobbyId;

#[derive(Resource)]
pub struct LobbyManager {
    lobby_room_keys: HashMap<LobbyId, RoomKey>,
}

impl Default for LobbyManager {
    fn default() -> Self {
        Self {
            lobby_room_keys: HashMap::new(),
        }
    }
}

impl LobbyManager {

    pub fn lobby_room_key(&self, lobby_id: &LobbyId) -> Option<RoomKey> {
        self.lobby_room_keys.get(lobby_id).cloned()
    }

    pub fn insert_lobby_room_key(&mut self, lobby_id: LobbyId, room_key: RoomKey) {
        self.lobby_room_keys.insert(lobby_id, room_key);
    }
}
