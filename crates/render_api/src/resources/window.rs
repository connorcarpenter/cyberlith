use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Window {
    pub resolution: Resolution,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            resolution: Resolution::new(width, height),
        }
    }
}

pub struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn physical_width(&self) -> u32 {
        self.width
    }

    pub fn physical_height(&self) -> u32 {
        self.height
    }
}
