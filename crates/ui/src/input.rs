use bevy_ecs::event::EventReader;
use bevy_log::{info, warn};

use input::{CursorIcon, GamepadButtonType, Input, InputEvent, Key, MouseButton};
use ui_layout::TextMeasurer;

use crate::{NodeId, Ui, UiEvent, WidgetKind};

pub struct UiInputConverter;

impl UiInputConverter {

    pub fn convert(input: &Input, input_events: &mut EventReader<InputEvent>) -> Option<UiInput> {
        let (output_mode, output_events) = Self::read_all(input_events);
        let Some(last_was_mouse) = output_mode else {
            return None;
        };
        if last_was_mouse {
            let mouse_pos = input.mouse_position();
            return Some(UiInput::Mouse(mouse_pos.x,
                                       mouse_pos.y,
                                       input.is_pressed(MouseButton::Left)));
        } else {
            if let Some(output_events) = output_events {
                return Some(UiInput::Events(output_events));
            } else {
                return None;
            }
        }
    }

    fn read_all(input_events: &mut EventReader<InputEvent>) -> (Option<bool>, Option<Vec<UiInputEvent>>) {
        let mut output_mode = None;
        let mut output_events = None;

        for input_event in input_events.read() {

            // first, check the mode of the input event
            if let Some(was_mouse) = match input_event {
                InputEvent::MouseClicked(_, _) | InputEvent::MouseReleased(_) | InputEvent::MouseMoved(_) | InputEvent::MouseDragged(_, _, _) | InputEvent::MouseMiddleScrolled(_) => {
                    Some(true)
                }
                InputEvent::Text(_) | InputEvent::KeyPressed(_, _) | InputEvent::KeyReleased(_) | InputEvent::GamepadButtonPressed(_, _) | InputEvent::GamepadButtonReleased(_, _) | InputEvent::GamepadJoystickMoved(_, _, _) => {
                    Some(false)
                }
                _ => None,
            } {
                if output_mode.is_none() {
                    output_mode = Some(was_mouse);
                }
                *output_mode.as_mut().unwrap() = was_mouse;
            }

            // then, collect the actual events
            let output_event = match input_event {
                InputEvent::GamepadButtonPressed(_, button) => {
                    match button {
                        GamepadButtonType::DPadUp => Some(UiInputEvent::Up),
                        GamepadButtonType::DPadDown => Some(UiInputEvent::Down),
                        GamepadButtonType::DPadLeft => Some(UiInputEvent::Left),
                        GamepadButtonType::DPadRight => Some(UiInputEvent::Right),
                        GamepadButtonType::Start | GamepadButtonType::South => Some(UiInputEvent::SelectPressed),
                        GamepadButtonType::East => Some(UiInputEvent::Back),
                        _ => None,
                    }
                }
                InputEvent::GamepadButtonReleased(_, button) => {
                    match button {
                        GamepadButtonType::Start | GamepadButtonType::South => Some(UiInputEvent::SelectReleased),
                        _ => None,
                    }
                }
                InputEvent::KeyPressed(key, _modifiers) => {
                    match key {
                        Key::ArrowUp => Some(UiInputEvent::Up),
                        Key::ArrowDown => Some(UiInputEvent::Down),
                        Key::ArrowLeft => Some(UiInputEvent::Left),
                        Key::ArrowRight => Some(UiInputEvent::Right),
                        Key::Enter => Some(UiInputEvent::SelectPressed),
                        Key::Escape => Some(UiInputEvent::Back),
                        Key::Backspace => Some(UiInputEvent::Backspace),
                        Key::Delete => Some(UiInputEvent::Delete),
                        Key::Home => Some(UiInputEvent::Home),
                        Key::End => Some(UiInputEvent::End),
                        _ => None,
                    }
                }
                InputEvent::KeyReleased(key) => {
                    match key {
                        Key::Enter => Some(UiInputEvent::SelectReleased),
                        _ => None,
                    }
                }
                InputEvent::Text(c) => Some(UiInputEvent::Key(*c)),
                _ => None,
            };
            let Some(output_event) = output_event else {
                continue;
            };

            if output_events.is_none() {
                output_events = Some(Vec::new());
            }
            output_events.as_mut().unwrap().push(output_event);
        };

        (output_mode, output_events)
    }
}

#[derive(Clone)]
pub enum UiInput {
    Mouse(f32, f32, bool),
    Events(Vec<UiInputEvent>)
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum UiInputEvent {
    Up, Down, Left, Right,
    SelectPressed, SelectReleased,
    Back, Backspace, Delete, Key(char), Home, End,
}

pub fn ui_receive_input(ui: &mut Ui, text_measurer: &dyn TextMeasurer, input: UiInput) {
    match input {
        UiInput::Mouse(x, y, left_pressed) => {
            ui.clear_hover();

            for node_id in 0..ui.store.nodes.len() {
                let node_id = NodeId::from_usize(node_id);
                ui_update_hover(ui, &node_id, x, y);
            }

            if left_pressed {
                if let Some(hover_node) = ui.get_hover() {
                    match ui.node_ref(&hover_node).unwrap().widget_kind() {
                        WidgetKind::Button => {
                            ui.set_selected_node(Some(hover_node));
                            ui.emit_event(&hover_node, UiEvent::Clicked);
                        }
                        WidgetKind::Textbox => {
                            ui.set_selected_node(Some(hover_node));
                            ui.reset_interact_timer();
                            let (_, height, posx, _, _) = ui.cache.bounds(&hover_node).unwrap();
                            ui.textbox_mut(&hover_node).unwrap().recv_click(text_measurer, x, posx, height);
                        }
                        _ => {}
                    }
                }
            } else {
                if let Some(selected_node) = ui.get_selected_node() {
                    match ui.node_ref(&selected_node).unwrap().widget_kind() {
                        WidgetKind::Button => {
                            ui.set_selected_node(None);
                        }
                        _ => {}
                    }
                }
            }
        }
        UiInput::Events(events) => {
            let mut hover_node = ui.get_hover();
            if hover_node.is_none() {
                let Some(first_input_id) = ui.get_first_input() else {
                    panic!("no first input set, cannot process input events without somewhere to start");
                };
                ui.receive_hover(&first_input_id);
                hover_node = Some(first_input_id);
            }
            let hover_node = hover_node.unwrap();
            let textbox_opt = {
                if let Some(selected_id) = ui.get_selected_node() {
                    if ui.node_ref(&selected_id).unwrap().widget_kind() == WidgetKind::Textbox {
                        Some(selected_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            for event in events {
                match event {
                    UiInputEvent::Up => {
                        if let Some(next_id) = ui.nav_get_up_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::Down => {
                        if let Some(next_id) = ui.nav_get_down_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::Left => {
                        if let Some(textbox_id) = textbox_opt {
                            ui.reset_interact_timer();
                            ui.textbox_mut(&textbox_id).unwrap().recv_input(event);
                        } else {
                            if let Some(next_id) = ui.nav_get_left_id(&hover_node) {
                                ui.receive_hover(&next_id);
                            }
                        }
                    }
                    UiInputEvent::Right => {
                        if let Some(textbox_id) = textbox_opt {
                            ui.reset_interact_timer();
                            ui.textbox_mut(&textbox_id).unwrap().recv_input(event);
                        } else {
                            if let Some(next_id) = ui.nav_get_right_id(&hover_node) {
                                ui.receive_hover(&next_id);
                            }
                        }
                    }
                    UiInputEvent::SelectPressed => {
                        match ui.node_ref(&hover_node).unwrap().widget_kind() {
                            WidgetKind::Button => {
                                ui.set_selected_node(Some(hover_node));
                                ui.emit_event(&hover_node, UiEvent::Clicked);
                            }
                            WidgetKind::Textbox => {
                                ui.reset_interact_timer();
                                ui.set_selected_node(Some(hover_node));
                            }
                            _ => {}
                        }
                    }
                    UiInputEvent::SelectReleased => {
                        if let Some(selected_node) = ui.get_selected_node() {
                            match ui.node_ref(&selected_node).unwrap().widget_kind() {
                                WidgetKind::Button => {
                                    ui.set_selected_node(None);
                                }
                                _ => {}
                            }
                        }
                    }
                    UiInputEvent::Back => {

                    }
                    UiInputEvent::Backspace | UiInputEvent::Delete | UiInputEvent::Key(_) | UiInputEvent::Home | UiInputEvent::End => {
                        if let Some(textbox_id) = textbox_opt {
                            ui.reset_interact_timer();
                            ui.textbox_mut(&textbox_id).unwrap().recv_input(event);
                        }
                    }
                }
            }
        }
    }
}

fn ui_update_hover(
    ui: &mut Ui,
    id: &NodeId,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some(node) = ui.store.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    if !node.visible {
        return;
    }

    let Some((width, height, child_offset_x, child_offset_y, _)) = ui.cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    match node.widget_kind() {
        WidgetKind::Button => {
            let Some(button_mut) = ui.store.button_mut(id) else {
                panic!("no button mut for node_id: {:?}", id);
            };
            if button_mut.mouse_is_inside(
                (width, height, child_offset_x, child_offset_y),
                mouse_x, mouse_y,
            ) {
                ui.receive_hover(id);
                ui.set_cursor_icon(CursorIcon::Hand);
            }
        }
        WidgetKind::Textbox => {
            let Some(textbox_mut) = ui.store.textbox_mut(id) else {
                panic!("no textbox mut for node_id: {:?}", id);
            };
            if textbox_mut.mouse_is_inside(
                (width, height, child_offset_x, child_offset_y),
                mouse_x, mouse_y,
            ) {
                ui.receive_hover(id);
                ui.set_cursor_icon(CursorIcon::Text);
            }
        }
        _ => {}
    }
}