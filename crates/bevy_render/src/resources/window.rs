use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Window {
    pub resolution: Resolution,
}

impl Window {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            resolution: Resolution::new(width, height),
        }
    }
}

pub struct Resolution {
    width: f32,
    height: f32,
}

impl Resolution {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn physical_width(&self) -> f32 {
        self.width
    }

    pub fn physical_height(&self) -> f32 {
        self.height
    }
}
