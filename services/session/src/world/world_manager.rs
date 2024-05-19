use std::{
    collections::HashMap,
    time::Duration,
};

use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use auth_server_types::UserId;
use bevy_http_client::ResponseKey as ClientResponseKey;

use region_server_http_proto::WorldConnectResponse;

#[derive(Resource)]
pub struct WorldManager {
    world_connect_response_keys: HashMap<ClientResponseKey<WorldConnectResponse>, UserKey>,
    world_connect_resend_rate: Duration,
    world_instances: HashMap<String, WorldInstanceData>,
}

impl WorldManager {
    pub fn new(
        world_connect_resend_rate: Duration,
    ) -> Self {
        Self {
            world_connect_response_keys: HashMap::new(),
            world_connect_resend_rate,
            world_instances: HashMap::new(),
        }
    }

    pub fn world_connect_resend_rate(&self) -> &Duration {
        &self.world_connect_resend_rate
    }

    // World Keys

    pub fn add_world_connect_response_key(
        &mut self,
        user_key: &UserKey,
        response_key: ClientResponseKey<WorldConnectResponse>,
    ) {
        self.world_connect_response_keys
            .insert(response_key, user_key.clone());
    }

    pub fn remove_world_connect_response_key(
        &mut self,
        response_key: &ClientResponseKey<WorldConnectResponse>,
    ) {
        self.world_connect_response_keys.remove(response_key);
    }

    pub fn world_connect_response_keys(
        &self,
    ) -> Vec<(ClientResponseKey<WorldConnectResponse>, UserKey)> {
        let mut out = Vec::new();
        for (res_key, usr_key) in self.world_connect_response_keys.iter() {
            out.push((res_key.clone(), *usr_key));
        }
        out
    }

    pub fn world_instance_exists(&self, world_instance_secret: &str) -> bool {
        self.world_instances.contains_key(world_instance_secret)
    }

    pub fn get_user_key_from_world_instance(
        &self,
        world_instance_secret: &str,
        user_id: &UserId,
    ) -> Option<UserKey> {
        let world_instance = self.world_instances.get(world_instance_secret)?;
        world_instance.user_id_to_key_map.get(user_id).copied()
    }

    pub fn world_set_user_connected(
        &mut self,
        user_key: &UserKey,
        world_instance_secret: &str,
        user_id: UserId,
    ) {
        if !self.world_instances.contains_key(world_instance_secret) {
            self.world_instances
                .insert(world_instance_secret.to_string(), WorldInstanceData::new());
        }
        let world_instance = self.world_instances.get_mut(world_instance_secret).unwrap();
        world_instance.add_user(*user_key, user_id);
    }
}

struct WorldInstanceData {
    user_id_to_key_map: HashMap<UserId, UserKey>,
}

impl WorldInstanceData {
    pub fn new() -> Self {
        Self {
            user_id_to_key_map: HashMap::new(),
        }
    }

    pub(crate) fn add_user(&mut self, user_key: UserKey, user_id: UserId) {
        self.user_id_to_key_map.insert(user_id, user_key);
    }
}