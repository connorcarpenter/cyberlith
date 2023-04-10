use render_api::{
    base::{AxisAlignedBoundingBox, Camera},
    RenderOperation, Transform,
};

use crate::{
    core::{ColorTexture, DepthTexture},
    renderer::{
        BaseMesh, Geometry, Light, Material, MaterialType, Mesh, Object, RenderCamera, RenderLight,
        RenderObject,
    },
};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: RenderCamera<'a>,
    pub lights: Vec<RenderLight<'a>>,
    pub objects: Vec<RenderObject<'a>>,
}

impl<'a> RenderPass<'a> {
    pub fn from_camera(
        camera: &'a Camera,
        transform: &'a Transform,
        operation: &'a RenderOperation,
    ) -> Self {
        Self {
            camera: RenderCamera::new(camera, transform, operation),
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn take(
        mut self,
    ) -> (
        RenderCamera<'a>,
        Vec<RenderLight<'a>>,
        Vec<RenderObject<'a>>,
    ) {
        (self.camera, self.lights, self.objects)
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }

    pub fn process_lights(render: &'a Vec<RenderLight<'a>>) -> Vec<&dyn Light> {
        render.iter().map(|light| light as &dyn Light).collect()
    }
}
