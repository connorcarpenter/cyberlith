use std::default::Default;

use bevy_ecs::prelude::{Entity, Resource};

pub struct OwnedEntity {
    pub confirmed: Entity,
}

impl OwnedEntity {
    pub fn new(confirmed_entity: Entity) -> Self {
        Self {
            confirmed: confirmed_entity,
        }
    }
}

#[derive(Resource)]
pub struct Global {
    pub owned_entity: Option<OwnedEntity>,
}

impl Default for Global {
    fn default() -> Self {
        Self { owned_entity: None }
    }
}
