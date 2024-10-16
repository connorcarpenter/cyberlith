use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

use crate::types::Direction;

#[derive(Component, Replicate)]
pub struct LookDirection {
    inner: Property<Direction>,
}

impl LookDirection {
    pub fn new(dir: Direction) -> Self {
        Self::new_complete(dir)
    }

    pub fn get(&self) -> Direction {
        *self.inner
    }

    pub fn set(&mut self, dir: Direction) {
        if *self.inner != dir {
            *self.inner = dir;
        }
    }
}
