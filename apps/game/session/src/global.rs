use std::collections::HashMap;
use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use bevy_http_client::{ResponseKey as ClientResponseKey};

use region_server_http_proto::{SessionRegisterInstanceResponse, WorldUserLoginResponse};

#[derive(Resource)]
pub struct Global {
    register_insance_response_key: Option<ClientResponseKey<SessionRegisterInstanceResponse>>,
    world_connect_response_keys: HashMap<ClientResponseKey<WorldUserLoginResponse>, UserKey>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            register_insance_response_key: None,
            world_connect_response_keys: HashMap::new(),
        }
    }
}

impl Global {

    pub fn register_insance_response_key(&self) -> Option<&ClientResponseKey<SessionRegisterInstanceResponse>> {
        self.register_insance_response_key.as_ref()
    }

    pub fn set_register_insance_response_key(&mut self, response_key: ClientResponseKey<SessionRegisterInstanceResponse>) {
        self.register_insance_response_key = Some(response_key);
    }

    pub fn add_world_key(&mut self, user_key: &UserKey, response_key: ClientResponseKey<WorldUserLoginResponse>) {
        self.world_connect_response_keys.insert(response_key, user_key.clone());
    }

    pub fn remove_world_key(&mut self, response_key: &ClientResponseKey<WorldUserLoginResponse>) {
        self.world_connect_response_keys.remove(response_key);
    }

    pub fn world_keys(&self) -> impl Iterator<Item = (&ClientResponseKey<WorldUserLoginResponse>, &UserKey)> {
        self.world_connect_response_keys.iter()
    }
}