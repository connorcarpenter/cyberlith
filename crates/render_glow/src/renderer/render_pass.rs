use std::collections::HashMap;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform},
    Handle,
};

use crate::renderer::{GpuMesh, Light, Material, RenderCamera, RenderObject};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: RenderCamera<'a>,
    pub lights: Vec<&'a dyn Light>,
    objects: HashMap<(Handle<CpuMesh>, Handle<CpuMaterial>), RenderObject<'a>>,
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
        mesh: &'a GpuMesh,
        mat: &'a dyn Material,
        transform: &'a Transform,
    ) {
        let key = (*mesh_handle, *mat_handle);
        if let Some(object) = self.objects.get_mut(&key) {
            object.add_transform(transform);
            return;
        } else {
            let mut object = RenderObject::new(mesh, mat);
            object.add_transform(transform);
            self.objects.insert(key, object);
            return;
        }
    }

    pub fn take(
        self,
    ) -> (
        RenderCamera<'a>,
        Vec<&'a dyn Light>,
        Vec<RenderObject<'a>>,
    ) {
        let objects: Vec<RenderObject<'a>> =
            self.objects.into_iter().map(|(_, object)| object).collect();
        (self.camera, self.lights, objects)
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }
}
