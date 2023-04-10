use std::default::Default;

use bevy_ecs::prelude::Resource;

use crate::components::Viewport;

#[derive(Resource, Default)]
pub struct Window {
    pub resolution: WindowResolution,
}

impl Window {
    pub fn width(&self) -> f32 {
        self.resolution.width()
    }

    pub fn height(&self) -> f32 {
        self.resolution.height()
    }

    pub fn physical_width(&self) -> u32 {
        self.resolution.physical_width()
    }

    pub fn physical_height(&self) -> u32 {
        self.resolution.physical_height()
    }

    pub fn scale_factor(&self) -> f64 {
        self.resolution.scale_factor()
    }

    pub fn viewport(&self) -> Viewport {
        Viewport::new_at_origin(self.physical_width(), self.physical_height())
    }
}

pub struct WindowResolution {
    physical_width: u32,
    physical_height: u32,
    scale_factor_override: Option<f64>,
    scale_factor: f64,
}

impl WindowResolution {
    pub fn new(logical_width: f32, logical_height: f32) -> Self {
        Self {
            physical_width: logical_width as u32,
            physical_height: logical_height as u32,
            ..Default::default()
        }
    }

    pub fn with_scale_factor_override(mut self, scale_factor_override: f64) -> Self {
        self.scale_factor_override = Some(scale_factor_override);
        self
    }

    pub fn width(&self) -> f32 {
        (self.physical_width() as f64 / self.scale_factor()) as f32
    }

    pub fn height(&self) -> f32 {
        (self.physical_height() as f64 / self.scale_factor()) as f32
    }

    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor_override
            .unwrap_or_else(|| self.base_scale_factor())
    }

    pub fn base_scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn scale_factor_override(&self) -> Option<f64> {
        self.scale_factor_override
    }

    pub fn set(&mut self, width: f32, height: f32) {
        self.set_physical_resolution(
            (width as f64 * self.scale_factor()) as u32,
            (height as f64 * self.scale_factor()) as u32,
        );
    }

    pub fn set_physical_resolution(&mut self, width: u32, height: u32) {
        self.physical_width = width;
        self.physical_height = height;
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        let (width, height) = (self.width(), self.height());
        self.scale_factor = scale_factor;
        self.set(width, height);
    }

    pub fn set_scale_factor_override(&mut self, scale_factor_override: Option<f64>) {
        let (width, height) = (self.width(), self.height());
        self.scale_factor_override = scale_factor_override;
        self.set(width, height);
    }
}

impl Default for WindowResolution {
    fn default() -> Self {
        WindowResolution {
            physical_width: 1280,
            physical_height: 720,
            scale_factor_override: None,
            scale_factor: 1.0,
        }
    }
}
