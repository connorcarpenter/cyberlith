use std::collections::HashMap;

use math::Mat4;
use render_api::{
    components::{Camera, TypedLight, Projection, Transform},
    resources::{RenderFrameContents, MaterialOrSkinHandle},
    base::CpuMesh,
};
use storage::Handle;

// Render Pass
pub struct RenderPass {
    pub camera: Camera,
    pub camera_transform: Transform,
    pub camera_projection: Projection,
    pub lights: Vec<TypedLight>,
    meshes: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
}

impl RenderPass {
    pub fn from_contents(
        contents: RenderFrameContents,
    ) -> Self {

        let RenderFrameContents {
            camera_opt,
            camera_transform_opt,
            camera_projection_opt,
            lights,
            meshes,
        } = contents;

        let Some(camera) = camera_opt else {
            panic!("RenderFrameContents missing camera");
        };
        let Some(camera_transform) = camera_transform_opt else {
            panic!("RenderFrameContents missing camera_transform");
        };
        let Some(camera_projection) = camera_projection_opt else {
            panic!("RenderFrameContents missing camera_projection");
        };

        Self {
            camera,
            camera_transform,
            camera_projection,
            lights,
            meshes,
        }
    }

    pub fn take(self) -> (Camera, Transform, Projection, Vec<TypedLight>, HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>) {
        (self.camera, self.camera_transform, self.camera_projection, self.lights, self.meshes)
    }
}
