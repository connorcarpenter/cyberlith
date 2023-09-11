use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use math::Vec3;

use crate::components::{
    ClearOperation, OrthographicProjection, Projection, RenderTarget, Transform, Viewport,
};

// Camera Bundle
#[derive(Default, Bundle)]
pub struct CameraBundle {
    pub camera: Camera,
    pub transform: Transform,
    pub projection: Projection,
}

impl CameraBundle {
    pub fn new_2d(viewport: &Viewport) -> Self {
        Self {
            camera: Camera {
                viewport: Some(*viewport),
                ..Default::default()
            },
            transform: Transform::from_xyz(
                viewport.width as f32 * 0.5,
                viewport.height as f32 * 0.5,
                1.0,
            )
            .looking_at(
                Vec3::new(
                    viewport.width as f32 * 0.5,
                    viewport.height as f32 * 0.5,
                    0.0,
                ),
                Vec3::NEG_Y,
            ),
            projection: Projection::Orthographic(OrthographicProjection {
                height: viewport.height as f32,
                near: 0.0,
                far: 10.0,
            }),
        }
    }
}

///
/// Represents a camera used for viewing 3D assets.
///
#[derive(Component, Clone, Copy)]
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
