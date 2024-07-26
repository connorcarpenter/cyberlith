use bevy_ecs::prelude::Component;

use game_engine::world::components::{NextTilePosition, PrevTilePosition};

#[derive(Component, Clone)]
pub struct BufferedNextTilePosition {
    x: i16,
    y: i16,
}

impl BufferedNextTilePosition {
    pub fn new(next_tile_position: &NextTilePosition) -> Self {
        let x = next_tile_position.x();
        let y = next_tile_position.y();

        Self { x, y }
    }

    pub fn x(&self) -> i16 {
        self.x
    }

    pub fn y(&self) -> i16 {
        self.y
    }

    pub fn equals(&self, ntp: &NextTilePosition) -> bool {
        self.x == ntp.x() && self.y == ntp.y()
    }

    pub fn incoming(&mut self, ptp: &mut PrevTilePosition, ntp: &NextTilePosition) {
        ptp.x = self.x;
        ptp.y = self.y;

        self.x = ntp.x();
        self.y = ntp.y();
    }
}
