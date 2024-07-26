use bevy_ecs::prelude::Component;

use game_engine::{naia::Tick, world::components::Position};

#[derive(Component)]
pub struct Interp {
    predicted: bool,
    last_tick: Tick,
    last_x: f32,
    last_y: f32,
    next_tick: Tick,
    next_x: f32,
    next_y: f32,
}

impl Interp {
    pub fn new(position: &Position) -> Self {
        let predicted = position.predicted();
        let tick = position.tick();
        let x = position.x();
        let y = position.y();
        Self {
            predicted,

            last_tick: tick,
            last_x: x,
            last_y: y,

            next_tick: tick,
            next_x: x,
            next_y: y,
        }
    }

    pub(crate) fn next_position(&mut self, position: &Position) {

        if position.predicted() != self.predicted {
            panic!("Interp.predicted != Position.predicted");
        }

        self.last_tick = self.next_tick;
        self.last_x = self.next_x;
        self.last_y = self.next_y;

        self.next_tick = position.tick();
        self.next_x = position.x();
        self.next_y = position.y();
    }

    pub(crate) fn interpolate(&self, interpolation: f32) -> (f32, f32) {
        let x = self.last_x + ((self.next_x - self.last_x) * interpolation);
        let y = self.last_y + ((self.next_y - self.last_y) * interpolation);

        // let x = self.next_x;
        // let y = self.next_y;

        return (x, y);
    }

    pub fn mirror(&mut self, other: &Self) {
        self.last_x = other.last_x;
        self.last_y = other.last_y;
        self.next_x = other.next_x;
        self.next_y = other.next_y;
    }
}
