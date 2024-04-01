use render_api::base::CpuMaterial;
use storage::Handle;

use crate::{button::NodeActiveState, panel::PanelState};

#[derive(Clone)]
pub struct TextboxState {
    pub panel: PanelState,

    pub text: String,

    hover_color_handle: Option<Handle<CpuMaterial>>,
    active_color_handle: Option<Handle<CpuMaterial>>,
    select_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextboxState {
    pub fn new() -> Self {
        Self {
            panel: PanelState::new(),
            text: String::new(),
            hover_color_handle: None,
            active_color_handle: None,
            select_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.panel.background_color_handle.is_none()
            || self.hover_color_handle.is_none()
            || self.active_color_handle.is_none()
            || self.select_color_handle.is_none()
    }

    pub fn current_color_handle(&self, state: NodeActiveState) -> Option<Handle<CpuMaterial>> {
        match state {
            NodeActiveState::Normal => self.panel.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.active_color_handle,
        }
    }

    pub fn set_hover_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.hover_color_handle = Some(val);
    }

    pub fn set_active_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.active_color_handle = Some(val);
    }

    pub fn get_selection_color_handle(&self) -> Option<Handle<CpuMaterial>> {
        self.select_color_handle
    }

    pub fn set_selection_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.select_color_handle = Some(val);
    }
}
