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
    objects: HashMap<Handle<CpuMaterial>, RenderObject>,
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
            objects: HashMap::new(),
        }
    }

    pub fn add_object(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &'a Transform,
    ) {
        if let Some(object) = self.objects.get_mut(mat_handle) {
            object.add_transform(mesh_handle, transform);
            return;
        } else {
            let mut object = RenderObject::new();
            object.add_transform(mesh_handle, transform);
            self.objects.insert(*mat_handle, object);
            return;
        }
    }

    pub fn take(self) -> (RenderCamera<'a>, Vec<&'a dyn Light>, HashMap<Handle<CpuMaterial>, RenderObject>) {
        (self.camera, self.lights, self.objects)
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }
}
