use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use auth_server_types::UserId;

#[derive(Component, Replicate)]
pub struct UserInfo {
    pub id: Property<UserId>,
    pub name: Property<String>,
}

impl UserInfo {
    pub fn new(
        id: UserId,
        name: &str,
    ) -> Self {
        Self::new_complete(id, name.to_string())
    }
}
