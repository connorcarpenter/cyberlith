
use unicode_segmentation::UnicodeSegmentation;

use input::{MouseButton};
use math::Vec2;
use render_api::base::CpuMaterial;
use storage::Handle;
use ui_layout::TextMeasurer;
use ui_types::Text;

use crate::{panel::PanelState, input::{UiInputEvent}, events::UiGlobalEvent, button::NodeActiveState};

#[derive(Clone)]
pub struct TextboxState {
    pub panel: PanelState,

    pub text: String,
    pub carat_index: usize,
    pub select_index: Option<usize>,

    hover_color_handle: Option<Handle<CpuMaterial>>,
    active_color_handle: Option<Handle<CpuMaterial>>,
    select_color_handle: Option<Handle<CpuMaterial>>,
}

impl TextboxState {
    pub fn new() -> Self {
        Self {
            panel: PanelState::new(),
            text: String::new(),
            carat_index: 0,
            select_index: None,
            hover_color_handle: None,
            active_color_handle: None,
            select_color_handle: None,
        }
    }

    pub fn needs_color_handle(&self) -> bool {
        self.panel.background_color_handle.is_none() || self.hover_color_handle.is_none() || self.active_color_handle.is_none() || self.select_color_handle.is_none()
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

    pub fn recv_keyboard_or_gamepad_event(&mut self, event: UiInputEvent) -> Option<Vec<UiGlobalEvent>> {
        let mut output = None;
        match event {
            UiInputEvent::LeftPressed(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if self.carat_index > 0 {
                            self.carat_index -= 1;
                        } else {
                            if output.is_none() {
                                output = Some(Vec::new());
                            }
                            output.as_mut().unwrap().push(UiGlobalEvent::PassThru);
                        }
                        self.select_index = None;

                    }
                    (true, false) => {
                        if self.carat_index > 0 {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }
                            self.carat_index -= 1;
                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                    (false, true) => {
                        if self.carat_index > 0 {
                            self.carat_index = self.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < self.carat_index)
                                .unwrap_or(0);
                        }
                        self.select_index = None;
                    }
                    (true, true) => {
                        if self.carat_index > 0 {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }

                            self.carat_index = self.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < self.carat_index)
                                .unwrap_or(0);

                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                }
            },
            UiInputEvent::RightPressed(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if self.carat_index < self.text.len() {
                            self.carat_index += 1;
                        } else {
                            if output.is_none() {
                                output = Some(Vec::new());
                            }
                            output.as_mut().unwrap().push(UiGlobalEvent::PassThru);
                        }
                        self.select_index = None;
                    }
                    (true, false) => {
                        if self.carat_index < self.text.len() {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }
                            self.carat_index += 1;
                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                    (false, true) => {
                        if self.carat_index < self.text.len() {
                            self.carat_index = self
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > self.carat_index)
                                .unwrap_or(self.text.len());
                        }
                        self.select_index = None;
                    }
                    (true, true) => {
                        if self.carat_index < self.text.len() {
                            if self.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                self.select_index = Some(self.carat_index);
                            }

                            self.carat_index = self
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > self.carat_index)
                                .unwrap_or(self.text.len());

                            if self.carat_index == self.select_index.unwrap() {
                                self.select_index = None;
                            }
                        }
                    }
                }
            },
            UiInputEvent::TextInsert(new_char) => {
                if let Some(select_index) = self.select_index {
                    // need to remove the selected text
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    self.text.replace_range(start..end, new_char.to_string().as_str());
                    self.carat_index = start + 1;
                    self.select_index = None;
                } else {
                    self.text.insert(self.carat_index, new_char);
                    self.carat_index += 1;
                }
            },
            UiInputEvent::BackspacePressed(modifiers) => {
                if let Some(select_index) = self.select_index {
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    self.text.drain(start..end);
                    self.carat_index = start;
                    self.select_index = None;
                } else {
                    if modifiers.ctrl {
                        if self.carat_index > 0 {
                            let target_index = self.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < self.carat_index)
                                .unwrap_or(0);
                            self.text.drain(target_index..self.carat_index);
                            self.carat_index = target_index;
                        }
                    } else {
                        if self.carat_index > 0 {
                            self.text.remove(self.carat_index - 1);
                            self.carat_index -= 1;
                        }
                    }
                }
            },
            UiInputEvent::DeletePressed(modifiers) => {
                if let Some(select_index) = self.select_index {
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    self.text.drain(start..end);
                    self.carat_index = start;
                    self.select_index = None;
                } else {
                    if modifiers.ctrl {
                        if self.carat_index < self.text.len() {
                            let target_index = self
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > self.carat_index)
                                .unwrap_or(self.text.len());
                            self.text.drain(self.carat_index..target_index);
                        }
                    } else {
                        if self.carat_index < self.text.len() {
                            self.text.remove(self.carat_index);
                        }
                    }
                }
            },
            UiInputEvent::HomePressed(modifiers) => {
                if modifiers.shift {
                    if self.select_index.is_none() {
                        self.select_index = Some(self.carat_index);
                    }
                    self.carat_index = 0;
                    if self.carat_index == self.select_index.unwrap() {
                        self.select_index = None;
                    }
                } else {
                    self.carat_index = 0;
                    self.select_index = None;
                }
            },
            UiInputEvent::EndPressed(modifiers) => {
                if modifiers.shift {
                    if self.select_index.is_none() {
                        self.select_index = Some(self.carat_index);
                    }
                    self.carat_index = self.text.len();
                    if self.carat_index == self.select_index.unwrap() {
                        self.select_index = None;
                    }
                } else {
                    self.carat_index = self.text.len();
                    self.select_index = None;
                }
            },
            UiInputEvent::TextCopy => {
                if let Some(select_index) = self.select_index {
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    let copied_text = self.text[start..end].to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output.as_mut().unwrap().push(UiGlobalEvent::Copied(copied_text));
                }
            }
            UiInputEvent::TextCut => {
                if let Some(select_index) = self.select_index {
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    let copied_text = self.text[start..end].to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output.as_mut().unwrap().push(UiGlobalEvent::Copied(copied_text));

                    self.text.drain(start..end);
                    self.carat_index = start;
                    self.select_index = None;
                }
            }
            UiInputEvent::TextPaste(text) => {
                // TODO: validate pasted text? I did panic at some point here.
                if let Some(select_index) = self.select_index {
                    let start = self.carat_index.min(select_index);
                    let end = self.carat_index.max(select_index);
                    self.text.replace_range(start..end, &text);
                    self.carat_index = start + text.len();
                    self.select_index = None;
                } else {
                    self.text.insert_str(self.carat_index, &text);
                    self.carat_index += text.len();
                }
            }
            UiInputEvent::TextSelectAll => {
                self.select_index = Some(0);
                self.carat_index = self.text.len();
            }
            _ => panic!("Unhandled input event for textbox: {:?}", event),
        }

        output
    }

    pub fn recv_mouse_event(
        &mut self,
        text_measurer: &dyn TextMeasurer,
        node_x: f32,
        node_h: f32,
        mouse_position_opt: Option<Vec2>,
        mouse_event: UiInputEvent,
    ) {
        match mouse_event {
            UiInputEvent::MouseSingleClick(MouseButton::Left, click_position, modifiers) => {
                if !modifiers.shift {
                    self.select_index = None;
                } else {
                    if self.select_index.is_none() {
                        self.select_index = Some(self.carat_index);
                    }
                }

                self.carat_index = Self::get_closest_index(&self.text, text_measurer, click_position.x, node_x, node_h);
                if let Some(select_index) = self.select_index {
                    if self.carat_index == select_index {
                        self.select_index = None;
                    }
                }
            }
            UiInputEvent::MouseDoubleClick(MouseButton::Left, click_position) => {
                // double click
                let click_index = Self::get_closest_index(&self.text, text_measurer, click_position.x, node_x, node_h);

                // select word
                let word_start = self.text
                    .unicode_word_indices()
                    .rev()
                    .map(|(i, _)| i)
                    .find(|&i| i < click_index)
                    .unwrap_or(0);
                let word_end = self
                    .text
                    .unicode_word_indices()
                    .map(|(i, word)| i + word.len())
                    .find(|&i| i > click_index)
                    .unwrap_or(self.text.len());

                self.select_index = Some(word_start);
                self.carat_index = word_end;
            }
            UiInputEvent::MouseTripleClick(MouseButton::Left, _) => {
                // triple click
                // select all
                self.select_index = Some(0);
                self.carat_index = self.text.len();
            }
            UiInputEvent::MouseButtonDrag(MouseButton::Left, modifiers) => {
                if let Some(mouse_position) = mouse_position_opt {
                    if modifiers.shift {
                        if self.select_index.is_none() {
                            self.select_index = Some(self.carat_index);
                        }
                        self.carat_index = Self::get_closest_index(&self.text, text_measurer, mouse_position.x, node_x, node_h);
                        if let Some(select_index) = self.select_index {
                            if self.carat_index == select_index {
                                self.select_index = None;
                            }
                        }
                    } else {
                        if let Some(select_index) = self.select_index {
                            self.carat_index = Self::get_closest_index(&self.text, text_measurer, mouse_position.x, node_x, node_h);
                            if self.carat_index == select_index {
                                self.select_index = None;
                            }
                        } else {
                            let new_index = Self::get_closest_index(&self.text, text_measurer, mouse_position.x, node_x, node_h);
                            if new_index != self.carat_index {
                                self.select_index = Some(self.carat_index);
                                self.carat_index = new_index;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_closest_index(text: &str, text_measurer: &dyn TextMeasurer, click_x: f32, position_x: f32, height: f32) -> usize {
        let click_x = click_x - position_x;

        let mut closest_x: f32 = f32::MAX;
        let mut closest_index: usize = usize::MAX;

        let subimage_indices = Text::get_subimage_indices(text);
        let (x_positions, text_height) = Text::get_raw_text_rects(text_measurer, &subimage_indices);
        let scale = height / text_height;

        for (char_index, x_position) in x_positions.iter().enumerate() {
            let index_x = 8.0 + (x_position * scale);
            let dist = (click_x - index_x).abs();
            if dist < closest_x {
                closest_x = dist;
                closest_index = char_index;
            } else {
                // dist is increasing ... we can break
                return closest_index;
            }
        }

        return closest_index;
    }
}