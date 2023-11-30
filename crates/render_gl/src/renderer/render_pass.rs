use std::collections::HashMap;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform},
    Handle,
};

use crate::{renderer::{Light, Material, RenderCamera, RenderObject}, GpuMesh};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: RenderCamera<'a>,
    pub lights: Vec<&'a dyn Light>,
    objects: Option<RenderObject>,
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
            objects: None,
        }
    }

    pub fn add_object(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &'a Transform,
    ) {
        if let Some(object) = self.objects.as_mut() {
            object.add_transform(mesh_handle, mat_handle, transform);
            return;
        } else {
            let mut object = RenderObject::new();
            object.add_transform(mesh_handle, mat_handle, transform);
            self.objects = Some(object);
            return;
        }
    }

    pub fn take(self) -> (RenderCamera<'a>, Vec<&'a dyn Light>, RenderObject) {
        (self.camera, self.lights, self.objects.unwrap())
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }
}
