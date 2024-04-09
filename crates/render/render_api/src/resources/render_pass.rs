use std::collections::HashMap;

use math::Mat4;
use storage::Handle;

use crate::{
    base::CpuMesh,
    components::{Camera, RenderLayer, Projection, Transform, TypedLight},
    resources::MaterialOrSkinHandle,
};

pub struct RenderPass {
    pub camera_opt: Option<Camera>,
    pub camera_transform_opt: Option<Transform>,
    pub camera_projection_opt: Option<Projection>,
    pub lights: Vec<TypedLight>,
    pub meshes: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
}

impl Default for RenderPass {
    fn default() -> Self {
        Self {
            camera_opt: None,
            camera_transform_opt: None,
            camera_projection_opt: None,
            lights: Vec::new(),
            meshes: HashMap::new(),
        }
    }
}

impl RenderPass {
    pub fn add_mesh(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: MaterialOrSkinHandle,
        transform_matrix: Mat4,
    ) {
        if !self.meshes.contains_key(mesh_handle) {
            self.meshes.insert(*mesh_handle, Vec::new());
        }
        let map = self.meshes.get_mut(mesh_handle).unwrap();
        map.push((mat_handle, transform_matrix));
    }
}
