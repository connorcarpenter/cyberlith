use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use three_d::{Camera as InnerCamera, ClearState};

use super::transform::Transform;
use crate::{
    assets::{ClearColorConfig, Handle, Image},
    math::Vec2,
};

// Camera
#[derive(Component)]
pub struct Camera {
    order: usize,
    pub clear_state: ClearState,
    pub inner: InnerCamera,
    pub target: RenderTarget,
}

//scale: 1.0,
//near: 0.0,
//far: 1000.0,
//viewport_origin: Vec2::new(0.5, 0.5),

impl Camera {
    pub(crate) const MAX_CAMERAS: usize = 32;

    pub fn new(order: usize, clear_state: ClearState, target: RenderTarget, inner: InnerCamera) -> Self {
        let mut new_self =
        Self {
            order: 0,
            clear_state,
            inner,
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