use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use crate::{components::{ClearOperation, Projection, RenderTarget, Transform, Viewport}};

// Camera Bundle
#[derive(Default, Bundle)]
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub projection: Projection,
}

///
/// Represents a camera used for viewing 3D assets.
///
#[derive(Component)]
pub struct Camera {
    pub viewport: Option<Viewport>,
    pub order: usize,
    pub clear_operation: ClearOperation,
    pub target: RenderTarget,
    pub is_active: bool,
}

impl Camera {
    pub const MAX_CAMERAS: usize = 32;

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn set_order(&mut self, order: usize) {
        if order > Self::MAX_CAMERAS {
            panic!("Camera order must be less than {}", Self::MAX_CAMERAS);
        }
        self.order = order;
    }

    pub fn viewport_or_default(&self) -> Viewport {
        self.viewport.unwrap_or_default()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            is_active: true,
            viewport: None,
            order: 0,
            clear_operation: ClearOperation::default(),
            target: RenderTarget::Screen,
        }
    }
}
