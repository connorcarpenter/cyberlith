use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

// This is networked

#[derive(Component, Replicate)]
pub struct HasMoveBuffered {
    buffered: Property<bool>,
}

impl HasMoveBuffered {
    pub fn new(buffered: bool) -> Self {
        Self::new_complete(buffered)
    }

    pub fn buffered(&self) -> bool {
        *self.buffered
    }

    pub fn set_buffered(&mut self, buffered: bool) {
        *self.buffered = buffered;
    }
}
