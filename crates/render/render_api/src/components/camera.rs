use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use math::Vec3;

use crate::components::{
    ClearOperation, OrthographicProjection, PerspectiveProjection, Projection, RenderTarget,
    Transform, Viewport,
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
                1000.0,
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
                near: 0.0,
                far: 2000.0,
            }),
        }
    }

    pub fn default_3d_perspective(viewport: &Viewport) -> Self {
        Self {
            camera: Camera {
                viewport: Some(*viewport),
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(100.0, 100.0, 100.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::PI / 4.0,
                near: 0.1,
                far: 2000.0,
            }),
            ..Default::default()
        }
    }

    pub fn default_3d_orthographic(viewport: &Viewport) -> Self {
        Self {
            camera: Camera {
                viewport: Some(*viewport),
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(100.0, 100.0, 100.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Z),
            projection: Projection::Orthographic(OrthographicProjection::new(0.0, 1000.0)),
            ..Default::default()
        }
    }
}

///
/// Represents a camera used for viewing 3D assets.
///
#[derive(Component, Clone, Copy)]
pub struct Camera {
    pub viewport: Option<Viewport>,
    pub clear_operation: ClearOperation,
    pub target: RenderTarget,
    pub is_active: bool,
}

impl Camera {
    pub const MAX_CAMERAS: usize = 32;

    pub fn viewport_or_default(&self) -> Viewport {
        self.viewport.unwrap_or_default()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            is_active: true,
            viewport: None,
            clear_operation: ClearOperation::default(),
            target: RenderTarget::Screen,
        }
    }
}
