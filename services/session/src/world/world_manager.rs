use std::collections::HashMap;

use bevy_ecs::system::Resource;

use naia_bevy_server::UserKey;

use auth_server_types::UserId;

#[derive(Resource)]
pub struct WorldManager {
    world_instances: HashMap<String, WorldInstanceData>,
}

impl Default for WorldManager {
    fn default() -> Self {
        Self {
            world_instances: HashMap::new(),
        }
    }
}

impl WorldManager {
    // World Keys

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
        user_id: &UserId,
        user_key: &UserKey,
        world_instance_secret: &str,
    ) {
        if !self.world_instances.contains_key(world_instance_secret) {
            self.world_instances
                .insert(world_instance_secret.to_string(), WorldInstanceData::new());
        }
        let world_instance = self.world_instances.get_mut(world_instance_secret).unwrap();
        world_instance.add_user(*user_key, *user_id);
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
