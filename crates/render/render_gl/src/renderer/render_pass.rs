use std::collections::HashMap;

use math::Mat4;
use render_api::{
    components::TypedLight,
    resources::{RenderFrameContents, MaterialOrSkinHandle},
    base::CpuMesh,
};
use storage::Handle;

use crate::renderer::RenderCamera;

// Render Pass
pub struct RenderPass {
    pub camera: RenderCamera,
    pub lights: Vec<TypedLight>,
    meshes: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
}

impl RenderPass {
    pub fn from_contents(
        contents: RenderFrameContents,
    ) -> Self {

        let RenderFrameContents {
            camera_opt,
            lights,
            meshes,
        } = contents;

        let Some((camera, transform, projection)) = camera_opt else {
            panic!("RenderFrameContents missing camera");
        };

        Self {
            camera: RenderCamera::new(camera, transform, projection),
            lights,
            meshes,
        }
    }

    pub fn take(self) -> (RenderCamera, Vec<TypedLight>, HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>) {
        (self.camera, self.lights, self.meshes)
    }
}
