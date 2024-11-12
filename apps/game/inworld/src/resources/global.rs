use std::default::Default;

use bevy_ecs::prelude::{Entity, Resource};

pub struct OwnedEntity {
    pub confirmed: Entity,
    pub predicted: Entity,
}

impl OwnedEntity {
    pub fn new(confirmed_entity: Entity, predicted_entity: Entity) -> Self {
        Self {
            confirmed: confirmed_entity,
            predicted: predicted_entity,
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
