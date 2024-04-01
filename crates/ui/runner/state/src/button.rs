use render_api::base::CpuMaterial;
use storage::Handle;

use crate::panel::PanelState;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeActiveState {
    Normal,
    Hover,
    Active,
}

#[derive(Clone)]
pub struct ButtonState {
    pub panel: PanelState,

    hover_color_handle: Option<Handle<CpuMaterial>>,
    down_color_handle: Option<Handle<CpuMaterial>>,
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            panel: PanelState::new(),
            hover_color_handle: None,
            down_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.panel.background_color_handle.is_none()
            || self.hover_color_handle.is_none()
            || self.down_color_handle.is_none()
    }

    pub fn current_color_handle(&self, state: NodeActiveState) -> Option<Handle<CpuMaterial>> {
        match state {
            NodeActiveState::Normal => self.panel.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.down_color_handle,
        }
    }

    pub fn set_hover_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.hover_color_handle = Some(val);
    }

    pub fn set_down_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.down_color_handle = Some(val);
    }
}
