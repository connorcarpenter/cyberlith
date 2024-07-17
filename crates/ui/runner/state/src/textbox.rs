use std::str::FromStr;

use ascii::{AsciiChar, AsciiString};
use unicode_segmentation::UnicodeSegmentation;

use render_api::base::CpuMaterial;
use storage::Handle;
use ui_runner_config::Textbox;

use crate::NodeActiveState;

#[derive(Clone)]
pub struct TextboxState {
    text: AsciiString,
    pub offset_index: usize,
    pub password_mask: bool,
    pub eye_hover: bool,
}

impl TextboxState {
    pub fn new(textbox: &Textbox) -> Self {
        Self {
            text: AsciiString::new(),
            offset_index: 0,
            password_mask: textbox.is_password,
            eye_hover: false,
        }
    }

    pub fn get_masked_text(&self) -> String {
        "*".repeat(self.text.len())
    }

    pub fn get_text_string(&self) -> String {
        self.text.to_string()
    }

    pub fn get_text_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn set_text(&mut self, val: &str) {
        self.text = AsciiString::from_str(val).unwrap();
    }

    pub fn text_len(&self) -> usize {
        self.text.len()
    }

    pub fn drain_text(&mut self, start: usize, end: usize) {
        ascii_string_drain(&mut self.text, start, end);
    }

    pub fn text_unicode_word_indices(&self) -> impl DoubleEndedIterator<Item = (usize, &str)> {
        self.text.as_str().unicode_word_indices()
    }

    pub fn text_remove_at(&mut self, index: usize) {
        let text_len = self.text.len();
        if index >= text_len {
            return;
        }
        let _ = self.text.remove(index);
    }

    pub fn get_text_range_as_string(&self, start: usize, end: usize) -> String {
        self.text[start..end].to_string()
    }

    pub fn get_text_range_as_str(&self, start: usize, end: usize) -> &str {
        &self.text[start..end].as_str()
    }

    pub fn text_replace_range(&mut self, start: usize, end: usize, new_text: &AsciiString) {
        ascii_string_replace_range(&mut self.text, start, end, new_text);
    }

    pub fn text_insert_str(&mut self, index: usize, text: &AsciiString) {
        self.text.insert_str(index, text);
    }

    pub fn text_insert_char(&mut self, index: usize, chara: AsciiChar) {
        self.text.insert(index, chara);
    }

    pub fn receive_hover(
        &mut self,
        config: &Textbox,
        layout: (f32, f32, f32, f32),
        mouse_x: f32,
        mouse_y: f32,
    ) -> bool {
        if !config.is_password {
            return false;
        }

        let (width, height, posx, posy) = layout;

        // compare to password eye rendering, should be the same
        let eye_left_x = posx + width - (height * 0.5 * 1.2) - (height * 0.5);

        if mouse_x >= eye_left_x
            && mouse_x <= posx + width
            && mouse_y >= posy
            && mouse_y <= posy + height
        {
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
    text_color_handle: Option<Handle<CpuMaterial>>,
    hover_color_handle: Option<Handle<CpuMaterial>>,
    active_color_handle: Option<Handle<CpuMaterial>>,
    select_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextboxStyleState {
    pub fn new() -> Self {
        Self {
            background_color_handle: None,
            text_color_handle: None,
            hover_color_handle: None,
            active_color_handle: None,
            select_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.background_color_handle.is_none()
            || self.text_color_handle.is_none()
            || self.hover_color_handle.is_none()
            || self.active_color_handle.is_none()
            || self.select_color_handle.is_none()
    }

    pub fn current_color_handle(&self, state: NodeActiveState) -> Option<Handle<CpuMaterial>> {
        match state {
            NodeActiveState::Normal => self.background_color_handle,
            NodeActiveState::Hover => self.hover_color_handle,
            NodeActiveState::Active => self.active_color_handle,
            NodeActiveState::Disabled => {
                panic!("Disabled state not implemented for TextboxStyleState")
            }
        }
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

fn ascii_string_drain(text: &mut AsciiString, start: usize, end: usize) {
    // Make sure that `start` and `end` is a valid range for the text
    let valid = start <= end && end <= text.len();

    if !valid {
        return;
    }

    // Convert AsciiString to Vec<AsciiChar>
    let mut chars: Vec<AsciiChar> = text.chars().collect();

    // Remove the specified range
    chars.drain(start..=end);

    // Convert back to AsciiString
    *text = AsciiString::from(chars);
}

fn ascii_string_replace_range(
    text: &mut AsciiString,
    start: usize,
    end: usize,
    new_text: &AsciiString,
) {
    // Make sure that `start` and `end` is a valid range for the text
    let valid = start <= end && end < text.len();

    if !valid {
        return;
    }

    // Convert AsciiString to Vec<AsciiChar>
    let mut chars: Vec<AsciiChar> = text.chars().collect();

    // Replace the specified range
    chars.splice(start..end, new_text.chars());

    // Convert back to AsciiString
    *text = AsciiString::from(chars);
}