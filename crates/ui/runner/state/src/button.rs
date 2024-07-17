
use render_api::base::CpuMaterial;
use storage::Handle;
use ui_runner_config::Button;

#[derive(Clone)]
pub struct ButtonState {
    pub enabled: bool,
}

impl ButtonState {
    pub fn new(button: &Button) -> Self {
        Self {
            enabled: button.enabled,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NodeActiveState {
    Normal,
    Hover,
    Active,
    Disabled,
}

#[derive(Clone)]
pub struct ButtonStyleState {
    background_color_handle: Option<Handle<CpuMaterial>>,
    hover_color_handle: Option<Handle<CpuMaterial>>,
    down_color_handle: Option<Handle<CpuMaterial>>,
    disabled_color_handle: Option<Handle<CpuMaterial>>,
}

impl ButtonStyleState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
            hover_color_handle: None,
            down_color_handle: None,
            disabled_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.background_color_handle.is_none()
            || self.hover_color_handle.is_none()
            || self.down_color_handle.is_none()
            || self.disabled_color_handle.is_none()
    }

    pub fn current_color_handle(&self, button_state: &ButtonState, mut node_active_state: NodeActiveState) -> Option<Handle<CpuMaterial>> {

        if !button_state.enabled {
            node_active_state = NodeActiveState::Disabled;
        }

        match node_active_state {
            NodeActiveState::Normal => self.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.down_color_handle,
            NodeActiveState::Disabled => self.disabled_color_handle,
        }
    }

    pub fn set_hover_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.hover_color_handle = Some(val);
    }

    pub fn set_down_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.down_color_handle = Some(val);
    }

    pub fn set_background_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.background_color_handle = Some(val);
    }

    pub fn set_disabled_color_handle(&mut self, val: Handle<CpuMaterial>) {
        self.disabled_color_handle = Some(val);
    }
}
