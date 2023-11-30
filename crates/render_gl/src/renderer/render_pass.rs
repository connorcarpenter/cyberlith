use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform},
    Handle,
};

use crate::renderer::{Light, RenderCamera, RenderObject};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: RenderCamera<'a>,
    pub lights: Vec<&'a dyn Light>,
    objects: RenderObject,
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
            objects: RenderObject::new(),
        }
    }

    pub fn add_object(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &'a Transform,
    ) {
        self.objects.add_transform(mesh_handle, mat_handle, transform);
    }

    pub fn take(self) -> (RenderCamera<'a>, Vec<&'a dyn Light>, RenderObject) {
        (self.camera, self.lights, self.objects)
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }
}
