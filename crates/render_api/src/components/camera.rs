use std::default::Default;

use bevy_ecs::component::Component;

use crate::{assets::Handle, base::Camera, base::Texture2D};


use bevy_ecs::bundle::Bundle;

use super::transform::Transform;

// Camera Bundle
#[derive(Default, Bundle)]
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub render: RenderOperation,
}

// Render Operation
#[derive(Component)]
pub struct RenderOperation {
    pub order: usize,
    pub clear_operation: ClearOperation,
    pub target: RenderTarget,
}

impl RenderOperation {
    pub const MAX_CAMERAS: usize = 32;

    pub fn new(
        order: usize,
        clear_operation: ClearOperation,
        target: RenderTarget,
    ) -> Self {
        let mut new_self = Self {
            order: 0,
            clear_operation,
            target,
        };
        new_self.set_order(order);

        new_self
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn set_order(&mut self, order: usize) {
        if order > Self::MAX_CAMERAS {
            panic!("Camera order must be less than {}", Self::MAX_CAMERAS);
        }
        self.order = order;
    }
}

impl Default for RenderOperation {
    fn default() -> Self {
        Self {
            order: 0,
            clear_operation: ClearOperation::default(),
            target: RenderTarget::Screen,
        }
    }
}

// Render Target
pub enum RenderTarget {
    Screen,
    Image(Handle<Texture2D>),
}

// Clear Operation
pub struct ClearOperation {
    pub red: Option<f32>,
    pub green: Option<f32>,
    pub blue: Option<f32>,
    pub alpha: Option<f32>,
    pub depth: Option<f32>,
}

impl ClearOperation {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            red: Some(r),
            green: Some(g),
            blue: Some(b),
            alpha: Some(a),
            depth: Some(1.0),
        }
    }

    pub const fn none() -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: None,
        }
    }
}

impl Default for ClearOperation {
    fn default() -> Self {
        Self {
            red: Some(0.0),
            green: Some(0.0),
            blue: Some(0.0),
            alpha: Some(1.0),
            depth: Some(1.0),
        }
    }
}
