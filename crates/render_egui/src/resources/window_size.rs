use bevy_ecs::system::Resource;

/// Stores physical size and scale factor, is used as a helper to calculate logical size.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub(crate) physical_width: f32,
    pub(crate) physical_height: f32,
    pub(crate) scale_factor: f32,
}

impl WindowSize {
    pub(crate) fn new(physical_width: f32, physical_height: f32, scale_factor: f32) -> Self {
        Self {
            physical_width,
            physical_height,
            scale_factor,
        }
    }

    fn width(&self) -> f32 {
        self.physical_width / self.scale_factor
    }

    fn height(&self) -> f32 {
        self.physical_height / self.scale_factor
    }
}
