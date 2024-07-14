use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct ChatMessageGlobal;

impl ChatMessageGlobal {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
