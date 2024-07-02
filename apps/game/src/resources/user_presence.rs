use std::collections::{HashMap};

use bevy_ecs::{
    entity::Entity,
    system::{Resource},
};

use game_engine::{
    auth::UserId,
};

#[derive(Resource)]
pub struct UserPresence {
    users: HashMap<UserId, Entity>,
}

impl Default for UserPresence {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserPresence {

    pub(crate) fn get_user_info_entity(&self, user_id: &UserId) -> Option<Entity> {
        self.users.get(user_id).cloned()
    }

    pub fn recv_user(
        &mut self,
        user_id: UserId,
        user_entity: Entity,
    ) {
        self.users.insert(user_id, user_entity);
    }
}
