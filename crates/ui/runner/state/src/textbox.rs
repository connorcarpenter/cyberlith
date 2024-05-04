use crate::NodeActiveState;
use render_api::base::CpuMaterial;
use storage::Handle;

#[derive(Clone)]
pub struct TextboxState {
    pub text: String,
    pub offset_index: usize,
}

impl TextboxState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            offset_index: 0,
        }
    }
}

#[derive(Clone)]
pub struct TextboxStyleState {
    background_color_handle: Option<Handle<CpuMaterial>>,
    hover_color_handle: Option<Handle<CpuMaterial>>,
    active_color_handle: Option<Handle<CpuMaterial>>,
    select_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextboxStyleState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
            hover_color_handle: None,
            active_color_handle: None,
            select_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.background_color_handle.is_none()
            || self.hover_color_handle.is_none()
            || self.active_color_handle.is_none()
            || self.select_color_handle.is_none()
    }

    pub fn current_color_handle(&self, state: NodeActiveState) -> Option<Handle<CpuMaterial>> {
        match state {
            NodeActiveState::Normal => self.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.active_color_handle,
        }
    }

    pub fn set_background_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.background_color_handle = Some(handle);
    }

    pub fn set_hover_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.hover_color_handle = Some(handle);
    }

    pub fn set_active_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.active_color_handle = Some(handle);
    }

    pub fn set_select_color_handle(&mut self, handle: Handle<CpuMaterial>) {
        self.select_color_handle = Some(handle);
    }

    pub fn select_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.select_color_handle
    }
}
