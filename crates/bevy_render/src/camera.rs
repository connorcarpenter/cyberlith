use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use crate::color::ClearColorConfig;
use crate::{Handle, Image, Transform};

#[derive(Bundle, Default)]
pub struct Camera3dBundle {
    pub camera_3d: Camera3d,
    pub camera: Camera,
    pub transform: Transform,
}

#[derive(Component, Default)]
pub struct Camera3d {
    pub clear_color: ClearColorConfig,
}

#[derive(Component, Default)]
pub struct Camera {
    pub order: i32,
    pub target: RenderTarget,
}

pub enum RenderTarget {
    Image(Handle<Image>),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Image(Handle::new())
    }
}