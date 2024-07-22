use std::{collections::HashMap, time::Duration};

use bevy_ecs::system::Resource;

use naia_bevy_server::{RoomKey};

use social_server_types::LobbyId;

use crate::resources::region_connection::RegionServerState;

#[derive(Resource)]
pub struct Global {
    instance_secret: String,
    pub region_server: RegionServerState,
    lobby_room_keys: HashMap<LobbyId, RoomKey>,
}

impl Global {
    pub fn new(
        instance_secret: &str,
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            region_server: RegionServerState::new(
                registration_resend_rate,
                region_server_disconnect_timeout,
            ),
            lobby_room_keys: HashMap::new(),
        }
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }

    pub fn disconnect_region_server(&mut self) {
        self.region_server.set_disconnected();
    }

    pub fn lobby_room_key(&self, lobby_id: &LobbyId) -> Option<RoomKey> {
        self.lobby_room_keys.get(lobby_id).cloned()
    }

    pub fn insert_lobby_room_key(&mut self, lobby_id: LobbyId, room_key: RoomKey) {
        self.lobby_room_keys.insert(lobby_id, room_key);
    }
}
