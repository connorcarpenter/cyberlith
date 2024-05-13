use render_api::base::CpuMaterial;
use storage::Handle;

#[derive(Clone)]
pub struct SpinnerStyleState {
    background_color_handle: Option<Handle<CpuMaterial>>,
    spinner_color_handle: Option<Handle<CpuMaterial>>,
}

impl SpinnerStyleState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
            spinner_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.background_color_handle.is_none() || self.spinner_color_handle.is_none()
    }

    pub fn background_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.background_color_handle
    }

    pub fn set_background_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.background_color_handle = Some(handle);
    }

    pub fn spinner_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.spinner_color_handle
    }

    pub fn set_spinner_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.spinner_color_handle = Some(handle);
    }
}