use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate};

// This is networked

#[derive(Component, Replicate)]
pub struct NextTilePosition {
    x: Property<i16>,
    y: Property<i16>,
}

impl NextTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_complete(x, y)
    }

    pub fn x(&self) -> i16 {
        *self.x
    }

    pub fn y(&self) -> i16 {
        *self.y
    }

    pub fn set_x(&mut self, x: i16) {
        *self.x = x;
    }

    pub fn set_y(&mut self, y: i16) {
        *self.y = y;
    }

    pub fn set(&mut self, x: i16, y: i16) {
        *self.x = x;
        *self.y = y;
    }
}
