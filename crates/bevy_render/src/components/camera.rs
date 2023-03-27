use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use super::transform::Transform;
use crate::{
    assets::{ClearColorConfig, Handle, Image},
    math::Vec2,
};

#[derive(Bundle, Default)]
pub struct Camera3dBundle {
    pub camera_3d: Camera3d,
    pub camera: Camera,
    pub transform: Transform,
    pub projection: Projection,
}

// Camera 3
#[derive(Component, Default)]
pub struct Camera3d {
    pub clear_color: ClearColorConfig,
}

// Camera
#[derive(Component, Default)]
pub struct Camera {
    pub order: i32,
    pub target: RenderTarget,
}

// Render Target
pub enum RenderTarget {
    Image(Handle<Image>),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Image(Handle::new(0))
    }
}

// Projection
#[derive(Component, Clone)]
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

impl From<PerspectiveProjection> for Projection {
    fn from(p: PerspectiveProjection) -> Self {
        Self::Perspective(p)
    }
}

impl From<OrthographicProjection> for Projection {
    fn from(p: OrthographicProjection) -> Self {
        Self::Orthographic(p)
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective(Default::default())
    }
}

#[derive(Component, Clone)]
pub struct PerspectiveProjection {
    /// The vertical field of view (FOV) in radians.
    ///
    /// Defaults to a value of π/4 radians or 45 degrees.
    pub fov: f32,

    /// The aspect ratio (width divided by height) of the viewing frustum.
    ///
    /// Bevy's [`camera_system`](crate::camera::camera_system) automatically
    /// updates this value when the aspect ratio of the associated window changes.
    ///
    /// Defaults to a value of `1.0`.
    pub aspect_ratio: f32,

    /// The distance from the camera in world units of the viewing frustum's near plane.
    ///
    /// Objects closer to the camera than this value will not be visible.
    ///
    /// Defaults to a value of `0.1`.
    pub near: f32,

    /// The distance from the camera in world units of the viewing frustum's far plane.
    ///
    /// Objects farther from the camera than this value will not be visible.
    ///
    /// Defaults to a value of `1000.0`.
    pub far: f32,
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        PerspectiveProjection {
            fov: std::f32::consts::PI / 4.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 1.0,
        }
    }
}

#[derive(Component, Clone)]
pub struct OrthographicProjection {
    /// The distance of the near clipping plane in world units.
    ///
    /// Objects closer than this will not be rendered.
    ///
    /// Defaults to `0.0`
    pub near: f32,
    /// The distance of the far clipping plane in world units.
    ///
    /// Objects further than this will not be rendered.
    ///
    /// Defaults to `1000.0`
    pub far: f32,
    /// Specifies the origin of the viewport as a normalized position from 0 to 1, where (0, 0) is the bottom left
    /// and (1, 1) is the top right. This determines where the camera's position sits inside the viewport.
    ///
    /// When the projection scales due to viewport resizing, the position of the camera, and thereby `viewport_origin`,
    /// remains at the same relative point.
    ///
    /// Consequently, this is pivot point when scaling. With a bottom left pivot, the projection will expand
    /// upwards and to the right. With a top right pivot, the projection will expand downwards and to the left.
    /// Values in between will caused the projection to scale proportionally on each axis.
    ///
    /// Defaults to `(0.5, 0.5)`, which makes scaling affect opposite sides equally, keeping the center
    /// point of the viewport centered.
    pub viewport_origin: Vec2,
    /// Scales the projection in world units.
    ///
    /// As scale increases, the apparent size of objects decreases, and vice versa.
    ///
    /// Defaults to `1.0`
    pub scale: f32,
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        OrthographicProjection {
            scale: 1.0,
            near: 0.0,
            far: 1000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
        }
    }
}
