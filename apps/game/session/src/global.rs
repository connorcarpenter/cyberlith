use std::collections::HashMap;
use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use bevy_http_client::{ResponseKey as ClientResponseKey};

use region_server_http_proto::WorldConnectResponse;

#[derive(Resource)]
pub struct Global {
    world_connect_response_keys: HashMap<ClientResponseKey<WorldConnectResponse>, UserKey>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            world_connect_response_keys: HashMap::new(),
        }
    }
}

impl Global {
    pub fn add_world_key(&mut self, user_key: &UserKey, response_key: ClientResponseKey<WorldConnectResponse>) {
        self.world_connect_response_keys.insert(response_key, user_key.clone());
    }

    pub fn remove_world_key(&mut self, response_key: &ClientResponseKey<WorldConnectResponse>) {
        self.world_connect_response_keys.remove(response_key);
    }

    pub fn world_keys(&self) -> impl Iterator<Item = (&ClientResponseKey<WorldConnectResponse>, &UserKey)> {
        self.world_connect_response_keys.iter()
    }
}