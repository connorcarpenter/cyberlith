use render_api::components::Projection;
use render_api::{
    base::AxisAlignedBoundingBox,
    components::{Camera, Transform},
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
        projection: &'a Projection,
    ) -> Self {
        Self {
            camera: RenderCamera::new(camera, transform, projection),
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
