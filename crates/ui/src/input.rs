use bevy_ecs::event::EventReader;
use bevy_log::warn;

use input::{CursorIcon, GamepadButtonType, Input, InputEvent, Key, Modifiers, MouseButton};
use ui_layout::TextMeasurer;

use crate::{NodeId, Ui, UiEvent, WidgetKind};

pub struct UiInputConverter;

impl UiInputConverter {

    pub fn convert(input: &Input, input_events: &mut EventReader<InputEvent>) -> Option<UiInput> {
        Self::read_all(input_events)
    }

    fn read_all(input_events: &mut EventReader<InputEvent>) -> Option<UiInput> {
        let mut output_mode = None;
        let mut output_events = None;
        let mut last_mouse_click = None;

        for input_event in input_events.read() {

            // first, check the mode of the input event
            if let Some(was_mouse) = match input_event {
                InputEvent::MouseClicked(button, pos, modifiers) => {
                    if *button == MouseButton::Left {
                        last_mouse_click = Some(UiInput::Mouse(pos.x, pos.y, true, modifiers.clone()));
                    }
                    Some(true)
                }
                InputEvent::MouseReleased(_) | InputEvent::MouseMoved(_) |
                InputEvent::MouseDragged(_, _, _) | InputEvent::MouseMiddleScrolled(_) => {
                    Some(true)
                }
                InputEvent::Text(_) | InputEvent::KeyPressed(_, _) | InputEvent::KeyReleased(_) |
                InputEvent::GamepadButtonPressed(_, _) | InputEvent::GamepadButtonReleased(_, _) |
                InputEvent::GamepadJoystickMoved(_, _, _) | InputEvent::Cut | InputEvent::Copy | InputEvent::Paste(_)  => {
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
                        GamepadButtonType::DPadLeft => Some(UiInputEvent::Left(Modifiers::default())),
                        GamepadButtonType::DPadRight => Some(UiInputEvent::Right(Modifiers::default())),
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
                InputEvent::KeyPressed(key, modifiers) => {
                    match key {
                        Key::ArrowUp => Some(UiInputEvent::Up),
                        Key::ArrowDown => Some(UiInputEvent::Down),
                        Key::ArrowLeft => Some(UiInputEvent::Left(*modifiers)),
                        Key::ArrowRight => Some(UiInputEvent::Right(*modifiers)),
                        Key::Enter => Some(UiInputEvent::SelectPressed),
                        Key::Escape => Some(UiInputEvent::Back),
                        Key::Backspace => Some(UiInputEvent::Backspace(*modifiers)),
                        Key::Delete => Some(UiInputEvent::Delete(*modifiers)),
                        Key::Home => Some(UiInputEvent::Home(*modifiers)),
                        Key::End => Some(UiInputEvent::End(*modifiers)),
                        _ => None,
                    }
                }
                InputEvent::KeyReleased(key) => {
                    match key {
                        Key::Enter => Some(UiInputEvent::SelectReleased),
                        _ => None,
                    }
                }
                InputEvent::Text(c) => Some(UiInputEvent::Text(*c)),
                InputEvent::Paste(text) => Some(UiInputEvent::Paste(text.clone())),
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

        let Some(last_was_mouse) = output_mode else {
            return None;
        };
        if last_was_mouse {
            return last_mouse_click;
        } else {
            if let Some(output_events) = output_events {
                return Some(UiInput::Events(output_events));
            } else {
                return None;
            }
        }
    }
}

#[derive(Clone)]
pub enum UiInput {
    Mouse(f32, f32, bool, Modifiers),
    Events(Vec<UiInputEvent>)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum UiInputEvent {
    Up, Down, Left(Modifiers), Right(Modifiers),
    SelectPressed, SelectReleased,
    Back, Backspace(Modifiers), Delete(Modifiers), Text(char), Home(Modifiers), End(Modifiers),
    Copy, Cut, Paste(String),
}

pub fn ui_receive_input(ui: &mut Ui, text_measurer: &dyn TextMeasurer, input: UiInput) {
    match input {
        UiInput::Mouse(x, y, left_pressed, modifiers) => {
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
                            ui.textbox_mut(&hover_node).unwrap().recv_click(text_measurer, x, posx, height, &modifiers);
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

            // hover default ui element if none is hovered (coming from mouse mode)
            if hover_node.is_none() {
                let Some(first_input_id) = ui.get_first_input() else {
                    panic!("no first input set, cannot process input events without somewhere to start");
                };
                ui.receive_hover(&first_input_id);
                hover_node = Some(first_input_id);
            }

            // pipe event into textbox, if one is selected
            if let Some(selected_id) = ui.get_selected_node() {
                if ui.node_ref(&selected_id).unwrap().widget_kind() == WidgetKind::Textbox {
                    let textbox_id = selected_id;
                    for event in &events {
                        match event {
                            UiInputEvent::Right(_) | UiInputEvent::Left(_) | UiInputEvent::Backspace(_) | UiInputEvent::Delete(_) |
                            UiInputEvent::Text(_) | UiInputEvent::Home(_) | UiInputEvent::End(_) | UiInputEvent::Paste(_)
                            => {
                                ui.reset_interact_timer();
                                ui.textbox_mut(&textbox_id).unwrap().recv_input(event.clone());
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // handle navigation of hover elements && buttons
            let hover_node = hover_node.unwrap();
            for event in &events {
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
                    UiInputEvent::Left(_) => {
                        if let Some(next_id) = ui.nav_get_left_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::Right(_) => {
                        if let Some(next_id) = ui.nav_get_right_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::SelectPressed => {
                        match ui.node_ref(&hover_node).unwrap().widget_kind() {
                            WidgetKind::Button => {
                                ui.set_selected_node(Some(hover_node));
                                ui.emit_event(&hover_node, UiEvent::Clicked);
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
                    _ => {}
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