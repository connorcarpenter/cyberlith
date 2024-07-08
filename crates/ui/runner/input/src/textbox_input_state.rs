use std::str::FromStr;

use ascii::{AsciiChar, AsciiString};
use unicode_segmentation::UnicodeSegmentation;

use asset_id::AssetId;
use input::{Modifiers, MouseButton};
use math::Vec2;
use ui_runner_config::{
    get_carat_offset_and_scale, text_get_raw_rects, text_get_subimage_indices, NodeId, TextMeasurer,
};
use ui_state::TextboxState;

use crate::{input::UiManagerTrait, UiGlobalEvent, UiInputEvent};

#[derive(Clone)]
pub struct TextboxInputState;

impl TextboxInputState {
    pub fn recv_keyboard_or_gamepad_event(
        ui_manager: &mut dyn UiManagerTrait,
        text_measurer: &dyn TextMeasurer,
        ui_asset_id: &AssetId,
        textbox_id: &NodeId,
        event: UiInputEvent,
    ) -> Option<Vec<UiGlobalEvent>> {
        let config = ui_manager
            .ui_config(ui_asset_id)
            .unwrap()
            .textbox_ref(textbox_id)
            .unwrap();
        let text_len = ui_manager
            .ui_state(ui_asset_id)
            .textbox_ref(&textbox_id)
            .unwrap()
            .text
            .len();
        let carat_index = ui_manager.ui_input_state().carat_index;
        let select_index = ui_manager.ui_input_state().select_index;

        let mut output = None;
        match event {
            UiInputEvent::LeftPressed(modifiers) => {
                ui_manager.ui_input_state_mut().set_left_pressed(modifiers);
                Self::handle_left(
                    ui_manager,
                    ui_asset_id,
                    textbox_id,
                    modifiers,
                    false,
                    &mut output,
                );
            }
            UiInputEvent::LeftHeld(modifiers) => {
                Self::handle_left(
                    ui_manager,
                    ui_asset_id,
                    textbox_id,
                    modifiers,
                    true,
                    &mut output,
                );
            }
            UiInputEvent::LeftReleased => {
                ui_manager.ui_input_state_mut().set_left_released();
            }
            UiInputEvent::RightPressed(modifiers) => {
                ui_manager.ui_input_state_mut().set_right_pressed(modifiers);
                Self::handle_right(
                    ui_manager,
                    ui_asset_id,
                    textbox_id,
                    modifiers,
                    false,
                    &mut output,
                );
            }
            UiInputEvent::RightHeld(modifiers) => {
                Self::handle_right(
                    ui_manager,
                    ui_asset_id,
                    textbox_id,
                    modifiers,
                    true,
                    &mut output,
                );
            }
            UiInputEvent::RightReleased => {
                ui_manager.ui_input_state_mut().set_right_released();
            }
            UiInputEvent::BackspacePressed(modifiers) => {
                if let Some(select_index) = select_index {
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    ascii_string_drain(
                        &mut ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                        start,
                        end,
                    );
                    ui_manager.ui_input_state_mut().carat_index = start;
                    ui_manager.ui_input_state_mut().select_index = None;
                } else {
                    if modifiers.ctrl {
                        if carat_index > 0 {
                            let target_index = unicode_word_indices(
                                &ui_manager
                                    .ui_state(ui_asset_id)
                                    .textbox_ref(&textbox_id)
                                    .unwrap()
                                    .text,
                            )
                            .rev()
                            .map(|(i, _)| i)
                            .find(|&i| i < carat_index)
                            .unwrap_or(0);
                            ascii_string_drain(
                                &mut ui_manager
                                    .ui_state_mut(ui_asset_id)
                                    .textbox_mut(&textbox_id)
                                    .unwrap()
                                    .text,
                                target_index,
                                carat_index,
                            );
                            ui_manager.ui_input_state_mut().carat_index = target_index;
                        }
                    } else {
                        if carat_index > 0 {
                            let _ = ui_manager
                                .ui_state_mut(ui_asset_id)
                                .textbox_mut(&textbox_id)
                                .unwrap()
                                .text
                                .remove(carat_index - 1);
                            ui_manager.ui_input_state_mut().carat_index -= 1;
                        }
                    }
                }
            }
            UiInputEvent::DeletePressed(modifiers) => {
                if let Some(select_index) = select_index {
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    ascii_string_drain(
                        &mut ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                        start,
                        end,
                    );
                    ui_manager.ui_input_state_mut().carat_index = start;
                    ui_manager.ui_input_state_mut().select_index = None;
                } else {
                    if modifiers.ctrl {
                        if carat_index < text_len {
                            let target_index = unicode_word_indices(
                                &ui_manager
                                    .ui_state(ui_asset_id)
                                    .textbox_ref(&textbox_id)
                                    .unwrap()
                                    .text,
                            )
                            .map(|(i, word)| i + word.len())
                            .find(|&i| i > carat_index)
                            .unwrap_or(text_len);
                            ascii_string_drain(
                                &mut ui_manager
                                    .ui_state_mut(ui_asset_id)
                                    .textbox_mut(&textbox_id)
                                    .unwrap()
                                    .text,
                                carat_index,
                                target_index,
                            );
                        }
                    } else {
                        if carat_index < text_len {
                            let _ = ui_manager
                                .ui_state_mut(ui_asset_id)
                                .textbox_mut(&textbox_id)
                                .unwrap()
                                .text
                                .remove(carat_index);
                        }
                    }
                }
            }
            UiInputEvent::HomePressed(modifiers) => {
                if modifiers.shift {
                    if select_index.is_none() {
                        ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                    }
                    ui_manager.ui_input_state_mut().carat_index = 0;
                    if carat_index == select_index.unwrap() {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                } else {
                    ui_manager.ui_input_state_mut().carat_index = 0;
                    ui_manager.ui_input_state_mut().select_index = None;
                }
            }
            UiInputEvent::EndPressed(modifiers) => {
                if modifiers.shift {
                    if select_index.is_none() {
                        ui_manager.ui_input_state_mut().select_index =
                            Some(ui_manager.ui_input_state().carat_index);
                    }
                    ui_manager.ui_input_state_mut().carat_index = text_len;
                    if carat_index == select_index.unwrap() {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                } else {
                    ui_manager.ui_input_state_mut().carat_index = text_len;
                    ui_manager.ui_input_state_mut().select_index = None;
                }
            }
            UiInputEvent::TextSelectAll => {
                ui_manager.ui_input_state_mut().select_index = Some(0);
                ui_manager.ui_input_state_mut().carat_index = text_len;
            }
            UiInputEvent::TextCopy => {
                if let Some(select_index) = select_index {
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    let copied_text = ui_manager
                        .ui_state(ui_asset_id)
                        .textbox_ref(&textbox_id)
                        .unwrap()
                        .text[start..end]
                        .to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output
                        .as_mut()
                        .unwrap()
                        .push(UiGlobalEvent::Copied(copied_text));
                }
            }
            UiInputEvent::TextCut => {
                if let Some(select_index) = select_index {
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    let copied_text = ui_manager
                        .ui_state(ui_asset_id)
                        .textbox_ref(&textbox_id)
                        .unwrap()
                        .text[start..end]
                        .to_string();
                    if output.is_none() {
                        output = Some(Vec::new());
                    }
                    output
                        .as_mut()
                        .unwrap()
                        .push(UiGlobalEvent::Copied(copied_text));

                    ascii_string_drain(
                        &mut ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                        start,
                        end,
                    );
                    ui_manager.ui_input_state_mut().carat_index = start;
                    ui_manager.ui_input_state_mut().select_index = None;
                }
            }
            UiInputEvent::TextPaste(text) => {
                let mut text = AsciiString::from_str(&text).unwrap();
                if let Some(validator) = &config.validation {
                    let final_length = text_len + text.len();
                    if final_length > validator.max_length() {
                        let chars_left = validator.max_length() - text_len;
                        let text_slice = &text[0..chars_left];
                        text = AsciiString::from(text_slice);
                    }
                    if !validator.allows_text(text.as_str()) {
                        return None;
                    }
                }

                // TODO: validate pasted text here more? I did panic at some point here.

                if let Some(select_index) = select_index {
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    ascii_string_replace_range(
                        &mut ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                        start,
                        end,
                        &text,
                    );
                    ui_manager.ui_input_state_mut().carat_index = start + text.len();
                    ui_manager.ui_input_state_mut().select_index = None;
                } else {
                    ui_manager
                        .ui_state_mut(ui_asset_id)
                        .textbox_mut(&textbox_id)
                        .unwrap()
                        .text
                        .insert_str(carat_index, &text);
                    ui_manager.ui_input_state_mut().carat_index += text.len();
                }
            }
            UiInputEvent::CharacterInsert(new_char) => {
                let new_char = AsciiChar::new(new_char);
                if let Some(validator) = &config.validation {
                    if text_len >= validator.max_length() {
                        return None;
                    }
                    if !validator.includes_char(new_char.as_char()) {
                        return None;
                    }
                }

                if let Some(select_index) = select_index {
                    // need to remove the selected text
                    let start = carat_index.min(select_index);
                    let end = carat_index.max(select_index);
                    let replace_text =
                        AsciiString::from_str(new_char.to_string().as_str()).unwrap();
                    ascii_string_replace_range(
                        &mut ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                        start,
                        end,
                        &replace_text,
                    );
                    ui_manager.ui_input_state_mut().carat_index = start + 1;
                    ui_manager.ui_input_state_mut().select_index = None;
                } else {
                    ui_manager
                        .ui_state_mut(ui_asset_id)
                        .textbox_mut(&textbox_id)
                        .unwrap()
                        .text
                        .insert(carat_index, new_char);
                    ui_manager.ui_input_state_mut().carat_index += 1;
                }
            }
            _ => panic!("Unhandled input event for textbox: {:?}", event),
        }

        if carat_index
            < ui_manager
                .ui_state(ui_asset_id)
                .textbox_ref(&textbox_id)
                .unwrap()
                .offset_index
        {
            ui_manager
                .ui_state_mut(ui_asset_id)
                .textbox_mut(&textbox_id)
                .unwrap()
                .offset_index = carat_index;
        } else {
            // move the text offset index if the carat is out of view
            let (textbox_w, textbox_h, _, _, _) = ui_manager
                .ui_state(ui_asset_id)
                .cache
                .bounds(&textbox_id)
                .unwrap();
            let textbox_w = textbox_w - 8.0; // padding

            loop {
                let (carat_offset, _) = get_carat_offset_and_scale(
                    text_measurer,
                    textbox_h,
                    ui_manager
                        .ui_state(ui_asset_id)
                        .textbox_ref(&textbox_id)
                        .unwrap()
                        .text
                        .as_str(),
                    ui_manager
                        .ui_state(ui_asset_id)
                        .textbox_ref(&textbox_id)
                        .unwrap()
                        .offset_index,
                    carat_index,
                );
                let carat_offset = carat_offset + 8.0; // padding

                if carat_offset > textbox_w {
                    ui_manager
                        .ui_state_mut(ui_asset_id)
                        .textbox_mut(&textbox_id)
                        .unwrap()
                        .offset_index += 1;
                } else {
                    break;
                }
            }
        }

        output
    }

    fn handle_left(
        ui_manager: &mut dyn UiManagerTrait,
        ui_asset_id: &AssetId,
        textbox_id: &NodeId,
        modifiers: Modifiers,
        held: bool,
        output: &mut Option<Vec<UiGlobalEvent>>,
    ) {
        let carat_index = ui_manager.ui_input_state().carat_index;
        let select_index = ui_manager.ui_input_state().select_index;

        match (modifiers.shift, modifiers.ctrl) {
            (false, false) => {
                if carat_index > 0 {
                    ui_manager.ui_input_state_mut().carat_index -= 1;
                } else {
                    if !held {
                        // if we are at the beginning of the text, pass the event through to navigate out of textbox
                        if output.is_none() {
                            *output = Some(Vec::new());
                        }
                        output.as_mut().unwrap().push(UiGlobalEvent::PassThru);

                        ui_manager.ui_input_state_mut().set_left_released();
                    }
                }
                ui_manager.ui_input_state_mut().select_index = None;
            }
            (true, false) => {
                if carat_index > 0 {
                    if select_index.is_none() {
                        // if there is no current selection, set it to the current carat index
                        ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                    }
                    ui_manager.ui_input_state_mut().carat_index -= 1;
                    if carat_index == select_index.unwrap() {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                }
            }
            (false, true) => {
                if carat_index > 0 {
                    let new_carat_index = unicode_word_indices(
                        &ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                    )
                    .rev()
                    .map(|(i, _)| i)
                    .find(|&i| i < carat_index)
                    .unwrap_or(0);
                    ui_manager.ui_input_state_mut().carat_index = new_carat_index;
                }
                ui_manager.ui_input_state_mut().select_index = None;
            }
            (true, true) => {
                if carat_index > 0 {
                    if select_index.is_none() {
                        // if there is no current selection, set it to the current carat index
                        ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                    }

                    let new_carat_index = unicode_word_indices(
                        &ui_manager
                            .ui_state_mut(ui_asset_id)
                            .textbox_mut(&textbox_id)
                            .unwrap()
                            .text,
                    )
                    .rev()
                    .map(|(i, _)| i)
                    .find(|&i| i < carat_index)
                    .unwrap_or(0);
                    ui_manager.ui_input_state_mut().carat_index = new_carat_index;

                    if carat_index == select_index.unwrap() {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                }
            }
        }
    }

    fn handle_right(
        ui_manager: &mut dyn UiManagerTrait,
        ui_asset_id: &AssetId,
        textbox_id: &NodeId,
        modifiers: Modifiers,
        held: bool,
        output: &mut Option<Vec<UiGlobalEvent>>,
    ) {
        let text_len = ui_manager
            .ui_state(ui_asset_id)
            .textbox_ref(&textbox_id)
            .unwrap()
            .text
            .len();

        match (modifiers.shift, modifiers.ctrl) {
            (false, false) => {
                if ui_manager.ui_input_state().carat_index < text_len {
                    ui_manager.ui_input_state_mut().carat_index += 1;
                } else {
                    if !held {
                        // if we are at the end of the text, pass the event through to navigate out of textbox
                        if output.is_none() {
                            *output = Some(Vec::new());
                        }
                        output.as_mut().unwrap().push(UiGlobalEvent::PassThru);

                        ui_manager.ui_input_state_mut().set_right_released();
                    }
                }
                ui_manager.ui_input_state_mut().select_index = None;
            }
            (true, false) => {
                if ui_manager.ui_input_state().carat_index < text_len {
                    if ui_manager.ui_input_state().select_index.is_none() {
                        // if there is no current selection, set it to the current carat index
                        ui_manager.ui_input_state_mut().select_index =
                            Some(ui_manager.ui_input_state().carat_index);
                    }
                    ui_manager.ui_input_state_mut().carat_index += 1;
                    if ui_manager.ui_input_state().carat_index
                        == ui_manager.ui_input_state().select_index.unwrap()
                    {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                }
            }
            (false, true) => {
                if ui_manager.ui_input_state().carat_index < text_len {
                    let new_carat_index = unicode_word_indices(
                        &ui_manager
                            .ui_state(ui_asset_id)
                            .textbox_ref(&textbox_id)
                            .unwrap()
                            .text,
                    )
                    .map(|(i, word)| i + word.len())
                    .find(|&i| i > ui_manager.ui_input_state().carat_index)
                    .unwrap_or(text_len);
                    ui_manager.ui_input_state_mut().carat_index = new_carat_index;
                }
                ui_manager.ui_input_state_mut().select_index = None;
            }
            (true, true) => {
                if ui_manager.ui_input_state().carat_index < text_len {
                    if ui_manager.ui_input_state().select_index.is_none() {
                        // if there is no current selection, set it to the current carat index
                        ui_manager.ui_input_state_mut().select_index =
                            Some(ui_manager.ui_input_state().carat_index);
                    }

                    let new_carat_index = unicode_word_indices(
                        &ui_manager
                            .ui_state(ui_asset_id)
                            .textbox_ref(&textbox_id)
                            .unwrap()
                            .text,
                    )
                    .map(|(i, word)| i + word.len())
                    .find(|&i| i > ui_manager.ui_input_state().carat_index)
                    .unwrap_or(text_len);
                    ui_manager.ui_input_state_mut().carat_index = new_carat_index;

                    if ui_manager.ui_input_state().carat_index
                        == ui_manager.ui_input_state().select_index.unwrap()
                    {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                }
            }
        }
    }

    pub fn recv_mouse_event(
        ui_manager: &mut dyn UiManagerTrait,
        text_measurer: &dyn TextMeasurer,
        node_x: f32,
        node_h: f32,
        mouse_position_opt: Option<Vec2>,
        hover_asset_id: &AssetId,
        hover_node_id: &NodeId,
        mouse_event: UiInputEvent,
    ) {
        let textbox_state = ui_manager
            .ui_state_mut(hover_asset_id)
            .store
            .textbox_mut(&hover_node_id)
            .unwrap();
        if textbox_state.eye_hover {
            Self::recv_mouse_event_eye(textbox_state, mouse_event)
        } else {
            Self::recv_mouse_event_text(
                ui_manager,
                text_measurer,
                node_x,
                node_h,
                mouse_position_opt,
                hover_asset_id,
                hover_node_id,
                mouse_event,
            );
        }
    }

    fn recv_mouse_event_text(
        ui_manager: &mut dyn UiManagerTrait,
        text_measurer: &dyn TextMeasurer,
        node_x: f32,
        node_h: f32,
        mouse_position_opt: Option<Vec2>,
        hover_asset_id: &AssetId,
        hover_node_id: &NodeId,
        mouse_event: UiInputEvent,
    ) {
        let text_len = ui_manager
            .ui_state(hover_asset_id)
            .store
            .textbox_ref(&hover_node_id)
            .unwrap()
            .text
            .len();
        let carat_index = ui_manager.ui_input_state().carat_index;
        let select_index = ui_manager.ui_input_state().select_index;

        match mouse_event {
            UiInputEvent::MouseSingleClick(MouseButton::Left, click_position, modifiers) => {
                if !modifiers.shift {
                    ui_manager.ui_input_state_mut().select_index = None;
                } else {
                    if select_index.is_none() {
                        ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                    }
                }

                ui_manager.ui_input_state_mut().carat_index = Self::get_closest_index(
                    &ui_manager
                        .ui_state(hover_asset_id)
                        .store
                        .textbox_ref(&hover_node_id)
                        .unwrap(),
                    text_measurer,
                    click_position.x,
                    node_x,
                    node_h,
                );
                if let Some(select_index) = select_index {
                    if carat_index == select_index {
                        ui_manager.ui_input_state_mut().select_index = None;
                    }
                }
            }
            UiInputEvent::MouseDoubleClick(MouseButton::Left, click_position) => {
                // double click
                let click_index = Self::get_closest_index(
                    &ui_manager
                        .ui_state(hover_asset_id)
                        .store
                        .textbox_ref(&hover_node_id)
                        .unwrap(),
                    text_measurer,
                    click_position.x,
                    node_x,
                    node_h,
                );

                // select word
                let word_start = unicode_word_indices(
                    &ui_manager
                        .ui_state(hover_asset_id)
                        .store
                        .textbox_ref(&hover_node_id)
                        .unwrap()
                        .text,
                )
                .rev()
                .map(|(i, _)| i)
                .find(|&i| i < click_index)
                .unwrap_or(0);
                let word_end = unicode_word_indices(
                    &ui_manager
                        .ui_state(hover_asset_id)
                        .store
                        .textbox_ref(&hover_node_id)
                        .unwrap()
                        .text,
                )
                .map(|(i, word)| i + word.len())
                .find(|&i| i > click_index)
                .unwrap_or(text_len);

                ui_manager.ui_input_state_mut().select_index = Some(word_start);
                ui_manager.ui_input_state_mut().carat_index = word_end;
            }
            UiInputEvent::MouseTripleClick(MouseButton::Left, _) => {
                // triple click
                // select all
                ui_manager.ui_input_state_mut().select_index = Some(0);
                ui_manager.ui_input_state_mut().carat_index = text_len;
            }
            UiInputEvent::MouseButtonDrag(MouseButton::Left, modifiers) => {
                if let Some(mouse_position) = mouse_position_opt {
                    if modifiers.shift {
                        if select_index.is_none() {
                            ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                        }
                        ui_manager.ui_input_state_mut().carat_index = Self::get_closest_index(
                            &ui_manager
                                .ui_state(hover_asset_id)
                                .store
                                .textbox_ref(&hover_node_id)
                                .unwrap(),
                            text_measurer,
                            mouse_position.x,
                            node_x,
                            node_h,
                        );
                        if let Some(select_index) = select_index {
                            if carat_index == select_index {
                                ui_manager.ui_input_state_mut().select_index = None;
                            }
                        }
                    } else {
                        if let Some(select_index) = select_index {
                            ui_manager.ui_input_state_mut().carat_index = Self::get_closest_index(
                                &ui_manager
                                    .ui_state(hover_asset_id)
                                    .store
                                    .textbox_ref(&hover_node_id)
                                    .unwrap(),
                                text_measurer,
                                mouse_position.x,
                                node_x,
                                node_h,
                            );
                            if carat_index == select_index {
                                ui_manager.ui_input_state_mut().select_index = None;
                            }
                        } else {
                            let new_index = Self::get_closest_index(
                                &ui_manager
                                    .ui_state(hover_asset_id)
                                    .store
                                    .textbox_ref(&hover_node_id)
                                    .unwrap(),
                                text_measurer,
                                mouse_position.x,
                                node_x,
                                node_h,
                            );
                            if new_index != carat_index {
                                ui_manager.ui_input_state_mut().select_index = Some(carat_index);
                                ui_manager.ui_input_state_mut().carat_index = new_index;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn recv_mouse_event_eye(textbox_state: &mut TextboxState, mouse_event: UiInputEvent) {
        match mouse_event {
            UiInputEvent::MouseSingleClick(MouseButton::Left, _click_position, _modifiers) => {
                // toggle password mask
                textbox_state.password_mask = !textbox_state.password_mask;
            }
            _ => {}
        }
    }

    fn get_closest_index(
        textbox_state: &TextboxState,
        text_measurer: &dyn TextMeasurer,
        click_x: f32,
        position_x: f32,
        height: f32,
    ) -> usize {
        let text = &textbox_state.text;
        let offset_index = textbox_state.offset_index;
        let text = &text[offset_index..text.len()];

        let click_x = click_x - position_x;

        let mut closest_x: f32 = f32::MAX;
        let mut closest_index: usize = usize::MAX;

        let subimage_indices = text_get_subimage_indices(text.as_str());
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
                return closest_index + offset_index;
            }
        }

        return closest_index + offset_index;
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
    let valid = start <= end && end <= text.len();

    if !valid {
        return;
    }

    // Convert AsciiString to Vec<AsciiChar>
    let mut chars: Vec<AsciiChar> = text.chars().collect();

    // Replace the specified range
    chars.splice(start..=end, new_text.chars());

    // Convert back to AsciiString
    *text = AsciiString::from(chars);
}

fn unicode_word_indices(text: &AsciiString) -> impl DoubleEndedIterator<Item = (usize, &str)> {
    text.as_str().unicode_word_indices()
}
