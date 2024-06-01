use asset_id::AssetId;
use asset_loader::{AssetHandle, IconData};
use render_api::base::CpuMesh;
use storage::Handle;

pub struct StateGlobals {
    box_mesh_handle_opt: Option<Handle<CpuMesh>>,
    text_icon_handle_opt: Option<AssetHandle<IconData>>,
    eye_icon_handle_opt: Option<AssetHandle<IconData>>,
}

impl StateGlobals {
    pub(crate) fn new() -> Self {
        Self {
            box_mesh_handle_opt: None,
            text_icon_handle_opt: None,
            eye_icon_handle_opt: None,
        }
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.box_mesh_handle_opt.as_ref()
    }

    pub fn set_box_mesh_handle(&mut self, handle: Handle<CpuMesh>) {
        self.box_mesh_handle_opt = Some(handle);
    }

    pub fn get_text_icon_handle(&self) -> Option<&AssetHandle<IconData>> {
        self.text_icon_handle_opt.as_ref()
    }

    pub fn set_text_icon_handle(&mut self, asset_id: AssetId) {
        self.text_icon_handle_opt = Some(AssetHandle::new(asset_id));
    }

    pub fn get_eye_icon_handle(&self) -> Option<&AssetHandle<IconData>> {
        self.eye_icon_handle_opt.as_ref()
    }

    pub fn set_eye_icon_handle(&mut self, asset_id: AssetId) {
        self.eye_icon_handle_opt = Some(AssetHandle::new(asset_id));
    }
}
