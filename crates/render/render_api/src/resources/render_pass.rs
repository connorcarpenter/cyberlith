use std::collections::HashMap;

use math::Mat4;
use storage::Handle;

use crate::{resources::MaterialOrSkinHandle, base::CpuMesh, components::{Camera, Projection, Transform, TypedLight}};

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