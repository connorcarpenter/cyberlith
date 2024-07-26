use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct Interp {
    last_x: f32,
    last_y: f32,
    next_x: f32,
    next_y: f32,
}

impl Interp {
    pub fn new(x: i16, y: i16) -> Self {
        let x = x as f32;
        let y = y as f32;
        Self {
            last_x: x,
            last_y: y,
            next_x: x,
            next_y: y,
        }
    }

    pub(crate) fn next_position(&mut self, next_x: f32, next_y: f32) {
        self.last_x = self.next_x;
        self.last_y = self.next_y;
        self.next_x = next_x;
        self.next_y = next_y;
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
