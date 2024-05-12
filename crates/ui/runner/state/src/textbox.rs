
use render_api::base::CpuMaterial;
use storage::Handle;
use ui_runner_config::Textbox;

use crate::NodeActiveState;

#[derive(Clone)]
pub struct TextboxState {
    pub text: String,
    pub offset_index: usize,
    pub password_mask: bool,
    pub eye_hover: bool,
}

impl TextboxState {
    pub fn new(textbox: &Textbox) -> Self {
        Self {
            text: String::new(),
            offset_index: 0,
            password_mask: textbox.is_password,
            eye_hover: false,
        }
    }

    pub fn get_masked_text(&self) -> String {
        "*".repeat(self.text.len())
    }

    pub fn receive_hover(&mut self, config: &Textbox, layout: (f32, f32, f32, f32), mouse_x: f32, mouse_y: f32) -> bool {
        if !config.is_password {
            return false;
        }

        let (width, height, posx, posy) = layout;

        // compare to password eye rendering, should be the same
        let eye_left_x = posx + width - (height * 0.5 * 1.2) - (height * 0.5);

        if mouse_x >= eye_left_x && mouse_x <= posx + width && mouse_y >= posy && mouse_y <= posy + height {
            self.eye_hover = true;
            return true;
        } else {
            self.eye_hover = false;
            return false;
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
