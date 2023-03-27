use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Window {
    pub resolution: Resolution,
}

pub struct Resolution {
    width: f32,
    height: f32,
}

impl Resolution {
    pub fn physical_width(&self) -> f32 {
        self.width
    }

    pub fn physical_height(&self) -> f32 {
        self.height
    }
}