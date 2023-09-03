use std::default::Default;

use bevy_ecs::system::Resource;

use crate::components::Viewport;

#[derive(Resource)]
pub struct Window {
    resolution: Option<WindowResolution>,
    did_change: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            resolution: None,
            did_change: false,
        }
    }
}

impl Window {
    pub fn get(&self) -> Option<&WindowResolution> {
        self.resolution.as_ref()
    }

    pub fn set(&mut self, resolution: WindowResolution) {
        self.resolution = Some(resolution);
        self.did_change = true;
    }

    pub fn did_change(&self) -> bool {
        self.did_change
    }

    pub fn clear_change(&mut self) {
        self.did_change = false;
    }
}

pub struct WindowResolution {
    /// Viewport of the window in physical pixels (not counting pixel ratio)
    pub physical_size: Viewport,

    /// Viewport of the window in logical pixels = size / pixel ratio
    pub logical_size: Viewport,

    /// Number of physical pixels for each logical pixel.
    pub device_pixel_ratio: f64,
}