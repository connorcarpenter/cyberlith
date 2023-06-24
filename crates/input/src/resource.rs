use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Input {}

impl Input {
    pub fn new() -> Self {
        Self {}
    }

    pub fn mouse_x(&self) -> f32 {
        0.0
    }

    pub fn mouse_y(&self) -> f32 {
        0.0
    }
}