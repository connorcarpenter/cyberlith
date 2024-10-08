use std::default::Default;

use bevy_ecs::prelude::{Entity, Resource};

use game_engine::{naia::CommandHistory, world::messages::KeyCommand};

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
    pub queued_command: Option<KeyCommand>,
    pub command_history: CommandHistory<KeyCommand>,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            owned_entity: None,
            queued_command: None,
            command_history: CommandHistory::default(),
        }
    }
}
