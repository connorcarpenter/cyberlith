use std::collections::HashMap;
use std::default::Default;

use bevy_ecs::system::Resource;

use render_api::base::CpuMesh;
use render_api::Handle;

use crate::renderer::GpuMesh;

#[derive(Resource)]
pub struct GpuMeshManager {
    assets: HashMap<Handle<CpuMesh>, GpuMesh>,
}

impl GpuMeshManager {
    pub fn insert(&mut self, handle: Handle<CpuMesh>, i12n: GpuMesh) {
        self.assets.insert(handle, i12n);
    }

    pub fn get(&self, handle: &Handle<CpuMesh>) -> Option<&GpuMesh> {
        self.assets.get(&handle)
    }

    pub fn remove(&mut self, handle: &Handle<CpuMesh>) -> Option<GpuMesh> {
        self.assets.remove(handle)
    }
}

impl Default for GpuMeshManager {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}
