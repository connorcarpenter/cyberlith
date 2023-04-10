use render_api::{
    base::AxisAlignedBoundingBox,
    components::{Camera, Projection, Transform},
};

use crate::{
    core::{ColorTexture, DepthTexture},
    renderer::{BaseMesh, Geometry, Light, Material, MaterialType, Mesh, Object},
};

// Render Camera
#[derive(Clone, Copy)]
pub struct RenderCamera<'a> {
    pub camera: &'a Camera,
    pub transform: &'a Transform,
    pub projection: &'a Projection,
}

impl<'a> RenderCamera<'a> {
    pub fn new(camera: &'a Camera, transform: &'a Transform, projection: &'a Projection) -> Self {
        Self {
            camera,
            transform,
            projection,
        }
    }
}
