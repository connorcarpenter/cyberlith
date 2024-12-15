use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use crate::types::Direction;

// This is networked

#[derive(Component, Replicate)]
pub struct NetworkedMoveBuffer {
    buffered: Property<Option<Direction>>,
}

impl NetworkedMoveBuffer {
    pub fn new() -> Self {
        Self::new_complete(None)
    }

    pub fn buffered(&self) -> Option<Direction> {
        *self.buffered
    }

    pub fn set_buffered(&mut self, buffered: Option<Direction>) {
        *self.buffered = buffered;
    }
}
