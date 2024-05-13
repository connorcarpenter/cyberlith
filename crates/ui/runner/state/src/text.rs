use render_api::base::CpuMaterial;
use storage::Handle;
use ui_runner_config::Text;

#[derive(Clone)]
pub struct TextState {
    pub text: String,
}

impl TextState {
    pub fn new(text: &Text) -> Self {
        Self {
            text: text.init_text.clone(),
        }
    }
}

#[derive(Clone)]
pub struct TextStyleState {
    background_color_handle: Option<Handle<CpuMaterial>>,
    text_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextStyleState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
            text_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.background_color_handle.is_none() || self.text_color_handle().is_none()
    }

    pub fn background_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.background_color_handle
    }

    pub fn set_background_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.background_color_handle = Some(handle);
    }

    pub fn text_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.text_color_handle
    }

    pub fn set_text_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.text_color_handle = Some(handle);
    }
}
