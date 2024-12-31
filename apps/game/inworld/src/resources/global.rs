use std::default::Default;

use bevy_ecs::prelude::{Entity, Resource};

#[derive(Resource)]
pub struct Global {
    pub owned_entity: Option<Entity>,
}

impl Default for Global {
    fn default() -> Self {
        Self { owned_entity: None }
    }
}
