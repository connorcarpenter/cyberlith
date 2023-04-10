use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};
use math::Mat4;

use crate::{
    assets::Handle,
    base::{AxisAlignedBoundingBox, Texture2D},
};

use super::transform::Transform;

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

    ///
    /// Returns whether or not the given bounding box is within the camera frustum.
    /// It returns false if it is fully outside and true if it is inside or intersects.
    ///
    pub fn in_frustum(&self, aabb: &AxisAlignedBoundingBox) -> bool {
        // TODO: implement this!
        true
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

///
/// Defines the part of the screen/render target that is rendered to.
/// All values should be given in physical pixels.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Viewport {
    /// The distance in pixels from the left edge of the screen/render target.
    pub x: i32,
    /// The distance in pixels from the bottom edge of the screen/render target.
    pub y: i32,
    /// The width of the viewport.
    pub width: u32,
    /// The height of the viewport.
    pub height: u32,
}

impl Viewport {
    ///
    /// New viewport which starts at origin (x and y are both zero).
    ///
    pub fn new_at_origin(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    ///
    /// Returns the aspect ratio of this viewport.
    ///
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    ///
    /// Returns the intersection between this and the other Viewport.
    ///
    pub fn intersection(&self, other: impl Into<Self>) -> Self {
        let other = other.into();
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width =
            (self.x + self.width as i32 - x).clamp(0, other.x + other.width as i32 - x) as u32;
        let height =
            (self.y + self.height as i32 - y).clamp(0, other.y + other.height as i32 - y) as u32;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1280,
            height: 720,
        }
    }
}

pub trait CameraProjection {
    fn projection_matrix(&self, viewport: &Viewport) -> Mat4;
    fn near(&self) -> f32;
    fn far(&self) -> f32;
}

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

impl CameraProjection for Projection {
    fn projection_matrix(&self, viewport: &Viewport) -> Mat4 {
        match self {
            Projection::Perspective(projection) => projection.projection_matrix(viewport),
            Projection::Orthographic(projection) => projection.projection_matrix(viewport),
        }
    }

    fn near(&self) -> f32 {
        match self {
            Projection::Perspective(projection) => projection.near(),
            Projection::Orthographic(projection) => projection.near(),
        }
    }

    fn far(&self) -> f32 {
        match self {
            Projection::Perspective(projection) => projection.far(),
            Projection::Orthographic(projection) => projection.far(),
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective(Default::default())
    }
}

/// A 3D camera projection in which distant objects appear smaller than close objects.
#[derive(Clone)]
pub struct PerspectiveProjection {
    /// The vertical field of view (FOV) in radians.
    ///
    /// Defaults to a value of π/4 radians or 45 degrees.
    pub fov: f32,

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

impl CameraProjection for PerspectiveProjection {
    fn projection_matrix(&self, viewport: &Viewport) -> Mat4 {
        Mat4::perspective_rh(self.fov, viewport.aspect(), self.near, self.far)
    }

    fn near(&self) -> f32 {
        self.near
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        PerspectiveProjection {
            fov: std::f32::consts::PI / 4.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

#[derive(Clone)]
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
}

impl CameraProjection for OrthographicProjection {
    fn projection_matrix(&self, viewport: &Viewport) -> Mat4 {
        Mat4::orthographic_rh(
            -0.5 * viewport.width as f32,
            0.5 * viewport.width as f32,
            -0.5 * viewport.height as f32,
            0.5 * viewport.height as f32,
            self.near,
            self.far,
        )
    }

    fn near(&self) -> f32 {
        self.near
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        OrthographicProjection {
            near: 0.0,
            far: 1000.0,
        }
    }
}
