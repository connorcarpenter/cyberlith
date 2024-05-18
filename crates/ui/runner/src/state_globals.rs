use render_api::base::CpuMesh;
use storage::Handle;

pub struct StateGlobals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
}

impl StateGlobals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
        }
    }

    pub fn set_box_mesh_handle(&mut self, handle: Handle<CpuMesh>) {
        self.box_mesh_handle_opt = Some(handle);
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.box_mesh_handle_opt.as_ref()
    }
}