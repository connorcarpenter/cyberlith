use std::default::Default;

use bevy_ecs::system::Resource;

#[derive(Resource)]
pub struct Time {
    elapsed: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
        }
    }
}

impl Time {
    pub fn set_elapsed(&mut self, elapsed: f32) {
        self.elapsed = elapsed;
    }

    pub fn get_elapsed(&self) -> f32 {
        self.elapsed
    }
}