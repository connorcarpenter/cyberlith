use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use super::transform::Transform;
use crate::{
    assets::{Handle, Image},
    math::Vec2,
};

// Camera
#[derive(Component)]
pub struct Camera {
    order: usize,
    pub clear_operation: ClearOperation,
    pub target: RenderTarget,
}

impl Camera {
    pub const MAX_CAMERAS: usize = 32;

    pub fn new(order: usize, clear_operation: ClearOperation, target: RenderTarget) -> Self {
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

// Render Target
pub enum RenderTarget {
    Screen,
    Image(Handle<Image>),
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
            green: Some(1.0),
            blue: Some(0.0),
            alpha: Some(1.0),
            depth: Some(1.0),
        }
    }
}

// for later
//InnerCamera::new_orthographic(
//                 Viewport::new_at_origo(width, height),
//                 vec3(5.0, 5.0, 5.0),
//                 vec3(0.0, 0.0, 0.0),
//                 vec3(0.0, 1.0, 0.0),
//                 height as f32,
//                 0.0,
//                 1000.0,
//             ),
