use bevy_ecs::prelude::Component;

use naia_bevy_shared::{Property, Replicate, SignedVariableInteger};

// This is networked

#[derive(Component, Replicate)]
pub struct NextTilePosition {
    x: Property<i16>,
    y: Property<i16>,
    velocity_x: Property<SignedVariableInteger<11>>,
    velocity_y: Property<SignedVariableInteger<11>>,
}

impl NextTilePosition {
    pub fn new(x: i16, y: i16) -> Self {
        Self::new_complete(
            x,
            y,
            SignedVariableInteger::new(0),
            SignedVariableInteger::new(0),
        )
    }

    pub fn x(&self) -> i16 {
        *self.x
    }

    pub fn y(&self) -> i16 {
        *self.y
    }

    pub fn velocity_x(&self) -> f32 {
        self.velocity_x.get() as f32 / 100.0
    }

    pub fn velocity_y(&self) -> f32 {
        self.velocity_y.get() as f32 / 100.0
    }

    pub fn set(&mut self, x: i16, y: i16, velocity_x: f32, velocity_y: f32) {
        *self.x = x;
        *self.y = y;

        *self.velocity_x = SignedVariableInteger::new((velocity_x * 100.0) as i128);
        *self.velocity_y = SignedVariableInteger::new((velocity_y * 100.0) as i128);
    }
}
