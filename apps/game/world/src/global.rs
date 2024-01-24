
use bevy_ecs::system::Resource;

use bevy_http_client::{ResponseKey as ClientResponseKey};

use region_server_http_proto::WorldRegisterInstanceResponse;

#[derive(Resource)]
pub struct Global {
    register_insance_response_key: Option<ClientResponseKey<WorldRegisterInstanceResponse>>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            register_insance_response_key: None,
        }
    }
}

impl Global {

    pub fn register_insance_response_key(&self) -> Option<&ClientResponseKey<WorldRegisterInstanceResponse>> {
        self.register_insance_response_key.as_ref()
    }

    pub fn set_register_insance_response_key(&mut self, response_key: ClientResponseKey<WorldRegisterInstanceResponse>) {
        self.register_insance_response_key = Some(response_key);
    }
}