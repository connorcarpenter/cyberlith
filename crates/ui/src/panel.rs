use render_api::base::CpuMaterial;
use storage::Handle;

#[derive(Clone)]
pub struct PanelState {
    pub background_color_handle: Option<Handle<CpuMaterial>>,
}

impl PanelState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
        }
    }
}