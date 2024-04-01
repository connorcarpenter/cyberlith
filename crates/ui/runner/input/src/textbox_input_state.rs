
use unicode_segmentation::UnicodeSegmentation;

use input::MouseButton;
use math::Vec2;

use ui_runner_config::{text_get_raw_rects, text_get_subimage_indices, TextMeasurer};
use ui_state::TextboxState;

use crate::{UiGlobalEvent, UiInputEvent, UiInputState};

#[derive(Clone)]
pub struct TextboxInputState;

impl TextboxInputState {

    pub fn recv_keyboard_or_gamepad_event(input_state: &mut UiInputState, textbox_state: &mut TextboxState, event: UiInputEvent) -> Option<Vec<UiGlobalEvent>> {
        let mut output = None;
        match event {
            UiInputEvent::LeftPressed(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if input_state.carat_index > 0 {
                            input_state.carat_index -= 1;
                        } else {
                            if output.is_none() {
                                output = Some(Vec::new());
                            }
                            output.as_mut().unwrap().push(UiGlobalEvent::PassThru);
                        }
                        input_state.select_index = None;

                    }
                    (true, false) => {
                        if input_state.carat_index > 0 {
                            if input_state.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                input_state.select_index = Some(input_state.carat_index);
                            }
                            input_state.carat_index -= 1;
                            if input_state.carat_index == input_state.select_index.unwrap() {
                                input_state.select_index = None;
                            }
                        }
                    }
                    (false, true) => {
                        if input_state.carat_index > 0 {
                            input_state.carat_index = textbox_state.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < input_state.carat_index)
                                .unwrap_or(0);
                        }
                        input_state.select_index = None;
                    }
                    (true, true) => {
                        if input_state.carat_index > 0 {
                            if input_state.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                input_state.select_index = Some(input_state.carat_index);
                            }

                            input_state.carat_index = textbox_state.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < input_state.carat_index)
                                .unwrap_or(0);

                            if input_state.carat_index == input_state.select_index.unwrap() {
                                input_state.select_index = None;
                            }
                        }
                    }
                }
            },
            UiInputEvent::RightPressed(modifiers) => {
                match (modifiers.shift, modifiers.ctrl) {
                    (false, false) => {
                        if input_state.carat_index < textbox_state.text.len() {
                            input_state.carat_index += 1;
                        } else {
                            if output.is_none() {
                                output = Some(Vec::new());
                            }
                            output.as_mut().unwrap().push(UiGlobalEvent::PassThru);
                        }
                        input_state.select_index = None;
                    }
                    (true, false) => {
                        if input_state.carat_index < textbox_state.text.len() {
                            if input_state.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                input_state.select_index = Some(input_state.carat_index);
                            }
                            input_state.carat_index += 1;
                            if input_state.carat_index == input_state.select_index.unwrap() {
                                input_state.select_index = None;
                            }
                        }
                    }
                    (false, true) => {
                        if input_state.carat_index < textbox_state.text.len() {
                            input_state.carat_index = textbox_state
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > input_state.carat_index)
                                .unwrap_or(textbox_state.text.len());
                        }
                        input_state.select_index = None;
                    }
                    (true, true) => {
                        if input_state.carat_index < textbox_state.text.len() {
                            if input_state.select_index.is_none() {
                                // if there is no current selection, set it to the current carat index
                                input_state.select_index = Some(input_state.carat_index);
                            }

                            input_state.carat_index = textbox_state
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > input_state.carat_index)
                                .unwrap_or(textbox_state.text.len());

                            if input_state.carat_index == input_state.select_index.unwrap() {
                                input_state.select_index = None;
                            }
                        }
                    }
                }
            },
            UiInputEvent::TextInsert(new_char) => {
                if let Some(select_index) = input_state.select_index {
                    // need to remove the selected text
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    textbox_state.text.replace_range(start..end, new_char.to_string().as_str());
                    input_state.carat_index = start + 1;
                    input_state.select_index = None;
                } else {
                    textbox_state.text.insert(input_state.carat_index, new_char);
                    input_state.carat_index += 1;
                }
            },
            UiInputEvent::BackspacePressed(modifiers) => {
                if let Some(select_index) = input_state.select_index {
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    textbox_state.text.drain(start..end);
                    input_state.carat_index = start;
                    input_state.select_index = None;
                } else {
                    if modifiers.ctrl {
                        if input_state.carat_index > 0 {
                            let target_index = textbox_state.text
                                .unicode_word_indices()
                                .rev()
                                .map(|(i, _)| i)
                                .find(|&i| i < input_state.carat_index)
                                .unwrap_or(0);
                            textbox_state.text.drain(target_index..input_state.carat_index);
                            input_state.carat_index = target_index;
                        }
                    } else {
                        if input_state.carat_index > 0 {
                            textbox_state.text.remove(input_state.carat_index - 1);
                            input_state.carat_index -= 1;
                        }
                    }
                }
            },
            UiInputEvent::DeletePressed(modifiers) => {
                if let Some(select_index) = input_state.select_index {
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    textbox_state.text.drain(start..end);
                    input_state.carat_index = start;
                    input_state.select_index = None;
                } else {
                    if modifiers.ctrl {
                        if input_state.carat_index < textbox_state.text.len() {
                            let target_index = textbox_state
                                .text
                                .unicode_word_indices()
                                .map(|(i, word)| i + word.len())
                                .find(|&i| i > input_state.carat_index)
                                .unwrap_or(textbox_state.text.len());
                            textbox_state.text.drain(input_state.carat_index..target_index);
                        }
                    } else {
                        if input_state.carat_index < textbox_state.text.len() {
                            textbox_state.text.remove(input_state.carat_index);
                        }
                    }
                }
            },
            UiInputEvent::HomePressed(modifiers) => {
                if modifiers.shift {
                    if input_state.select_index.is_none() {
                        input_state.select_index = Some(input_state.carat_index);
                    }
                    input_state.carat_index = 0;
                    if input_state.carat_index == input_state.select_index.unwrap() {
                        input_state.select_index = None;
                    }
                } else {
                    input_state.carat_index = 0;
                    input_state.select_index = None;
                }
            },
            UiInputEvent::EndPressed(modifiers) => {
                if modifiers.shift {
                    if input_state.select_index.is_none() {
                        input_state.select_index = Some(input_state.carat_index);
                    }
                    input_state.carat_index = textbox_state.text.len();
                    if input_state.carat_index == input_state.select_index.unwrap() {
                        input_state.select_index = None;
                    }
                } else {
                    input_state.carat_index = textbox_state.text.len();
                    input_state.select_index = None;
                }
            },
            UiInputEvent::TextCopy => {
                if let Some(select_index) = input_state.select_index {
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    let copied_text = textbox_state.text[start..end].to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output.as_mut().unwrap().push(UiGlobalEvent::Copied(copied_text));
                }
            }
            UiInputEvent::TextCut => {
                if let Some(select_index) = input_state.select_index {
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    let copied_text = textbox_state.text[start..end].to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output.as_mut().unwrap().push(UiGlobalEvent::Copied(copied_text));

                    textbox_state.text.drain(start..end);
                    input_state.carat_index = start;
                    input_state.select_index = None;
                }
            }
            UiInputEvent::TextPaste(text) => {
                // TODO: validate pasted text? I did panic at some point here.
                if let Some(select_index) = input_state.select_index {
                    let start = input_state.carat_index.min(select_index);
                    let end = input_state.carat_index.max(select_index);
                    textbox_state.text.replace_range(start..end, &text);
                    input_state.carat_index = start + text.len();
                    input_state.select_index = None;
                } else {
                    textbox_state.text.insert_str(input_state.carat_index, &text);
                    input_state.carat_index += text.len();
                }
            }
            UiInputEvent::TextSelectAll => {
                input_state.select_index = Some(0);
                input_state.carat_index = textbox_state.text.len();
            }
            _ => panic!("Unhandled input event for textbox: {:?}", event),
        }

        output
    }

    pub fn recv_mouse_event(
        input_state: &mut UiInputState,
        text_measurer: &dyn TextMeasurer,
        textbox_state: &mut TextboxState,
        node_x: f32,
        node_h: f32,
        mouse_position_opt: Option<Vec2>,
        mouse_event: UiInputEvent,
    ) {
        match mouse_event {
            UiInputEvent::MouseSingleClick(MouseButton::Left, click_position, modifiers) => {
                if !modifiers.shift {
                    input_state.select_index = None;
                } else {
                    if input_state.select_index.is_none() {
                        input_state.select_index = Some(input_state.carat_index);
                    }
                }

                input_state.carat_index = Self::get_closest_index(&textbox_state.text, text_measurer, click_position.x, node_x, node_h);
                if let Some(select_index) = input_state.select_index {
                    if input_state.carat_index == select_index {
                        input_state.select_index = None;
                    }
                }
            }
            UiInputEvent::MouseDoubleClick(MouseButton::Left, click_position) => {
                // double click
                let click_index = Self::get_closest_index(&textbox_state.text, text_measurer, click_position.x, node_x, node_h);

                // select word
                let word_start = textbox_state.text
                    .unicode_word_indices()
                    .rev()
                    .map(|(i, _)| i)
                    .find(|&i| i < click_index)
                    .unwrap_or(0);
                let word_end = textbox_state
                    .text
                    .unicode_word_indices()
                    .map(|(i, word)| i + word.len())
                    .find(|&i| i > click_index)
                    .unwrap_or(textbox_state.text.len());

                input_state.select_index = Some(word_start);
                input_state.carat_index = word_end;
            }
            UiInputEvent::MouseTripleClick(MouseButton::Left, _) => {
                // triple click
                // select all
                input_state.select_index = Some(0);
                input_state.carat_index = textbox_state.text.len();
            }
            UiInputEvent::MouseButtonDrag(MouseButton::Left, modifiers) => {
                if let Some(mouse_position) = mouse_position_opt {
                    if modifiers.shift {
                        if input_state.select_index.is_none() {
                            input_state.select_index = Some(input_state.carat_index);
                        }
                        input_state.carat_index = Self::get_closest_index(&textbox_state.text, text_measurer, mouse_position.x, node_x, node_h);
                        if let Some(select_index) = input_state.select_index {
                            if input_state.carat_index == select_index {
                                input_state.select_index = None;
                            }
                        }
                    } else {
                        if let Some(select_index) = input_state.select_index {
                            input_state.carat_index = Self::get_closest_index(&textbox_state.text, text_measurer, mouse_position.x, node_x, node_h);
                            if input_state.carat_index == select_index {
                                input_state.select_index = None;
                            }
                        } else {
                            let new_index = Self::get_closest_index(&textbox_state.text, text_measurer, mouse_position.x, node_x, node_h);
                            if new_index != input_state.carat_index {
                                input_state.select_index = Some(input_state.carat_index);
                                input_state.carat_index = new_index;
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

        let subimage_indices = text_get_subimage_indices(text);
        let (x_positions, text_height) = text_get_raw_rects(text_measurer, &subimage_indices);
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