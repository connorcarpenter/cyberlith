use bevy_ecs::{system::Resource};

#[derive(Resource)]
pub struct Global {
}

impl Global {
    pub fn new() -> Self {
        Self {
        }
    }
}