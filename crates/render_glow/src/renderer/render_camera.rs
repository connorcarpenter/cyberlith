use render_api::{
    base::{AxisAlignedBoundingBox, Camera},
    components::Transform,
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
}

impl<'a> RenderCamera<'a> {
    pub fn new(
        camera: &'a Camera,
        transform: &'a Transform,
    ) -> Self {
        Self {
            camera,
            transform,
        }
    }
}
