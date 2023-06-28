use bevy_ecs::component::Component;

use math::Mat4;

use crate::components::Viewport;

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
    pub height: f32,
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
        let aspect_ratio = viewport.aspect();
        let width = self.height * aspect_ratio;
        let height = self.height;
        Mat4::orthographic_rh(
            -0.5 * width as f32,
            0.5 * width as f32,
            -0.5 * height as f32,
            0.5 * height as f32,
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
            height: 500.0,
            near: 0.0,
            far: 1000.0,
        }
    }
}
