use bevy_ecs::prelude::Component;

use naia_bevy_shared::{EntityProperty, Replicate};

#[derive(Component, Replicate)]
pub struct SelfhoodUser {
    pub user_entity: EntityProperty,
}

impl SelfhoodUser {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
