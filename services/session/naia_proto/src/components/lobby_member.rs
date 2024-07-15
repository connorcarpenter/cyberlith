use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Replicate};

#[derive(Component, Replicate)]
pub struct LobbyMember {
    pub lobby_entity: EntityProperty,
    pub user_entity: EntityProperty,
}

impl LobbyMember {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
