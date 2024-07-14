use bevy_ecs::prelude::Component;

use naia_bevy_shared::Replicate;

#[derive(Component, Replicate)]
pub struct MessageGlobal;

impl MessageGlobal {
    pub fn new() -> Self {
        Self::new_complete()
    }
}
