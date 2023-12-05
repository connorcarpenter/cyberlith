use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, Projection, Transform},
    Handle,
};
use render_api::resources::MaterialOrSkinHandle;

use crate::renderer::{Light, RenderCamera, RenderMeshes};

// Render Pass
pub struct RenderPass<'a> {
    pub camera: RenderCamera<'a>,
    pub lights: Vec<&'a dyn Light>,
    meshes: RenderMeshes,
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
            meshes: RenderMeshes::new(),
        }
    }

    pub fn add_mesh(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &MaterialOrSkinHandle,
        transform: &'a Transform,
    ) {
        self.meshes.add_instance(mesh_handle, mat_handle, transform);
    }

    pub fn take(self) -> (RenderCamera<'a>, Vec<&'a dyn Light>, RenderMeshes) {
        (self.camera, self.lights, self.meshes)
    }

    pub fn process_camera(render: &RenderCamera<'a>) -> &'a Camera {
        &render.camera
    }
}
