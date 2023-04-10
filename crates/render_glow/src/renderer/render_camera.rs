use render_api::{
    base::{AxisAlignedBoundingBox, Camera},
    RenderOperation, Transform,
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
    pub operation: &'a RenderOperation,
}

impl<'a> RenderCamera<'a> {
    pub fn new(
        camera: &'a Camera,
        transform: &'a Transform,
        operation: &'a RenderOperation,
    ) -> Self {
        Self {
            camera,
            transform,
            operation,
        }
    }
}
