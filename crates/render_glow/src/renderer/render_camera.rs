use render_api::{base::{AxisAlignedBoundingBox, Camera}, CameraComponent, Transform};

use crate::{core::{ColorTexture, DepthTexture}, renderer::{BaseMesh, Geometry, Light, Material, MaterialType, Mesh, Object, PostMaterial}};

// Render Camera
#[derive(Clone, Copy)]
pub struct RenderCamera<'a> {
    pub camera: &'a CameraComponent,
    // pub transform: &'a Transform, // later!
}

impl<'a> RenderCamera<'a> {
    pub fn new(camera: &'a CameraComponent) -> Self {
        Self {
            camera
        }
    }
}