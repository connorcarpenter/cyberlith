use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Replicate};

#[derive(Component, Replicate)]
pub struct ChatMessageLocal {
    pub owner_lobby_entity: EntityProperty,
}

impl ChatMessageLocal {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
