use render_api::base::CpuMaterial;
use storage::Handle;

#[derive(Clone)]
pub struct TextState {
    pub background_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
        }
    }
}
