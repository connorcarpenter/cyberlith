use bevy_ecs::event::EventReader;
use bevy_log::warn;

use input::{CursorIcon, GamepadButtonType, Input, InputEvent, Key, MouseButton};

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
                InputEvent::KeyPressed(_) | InputEvent::KeyReleased(_) | InputEvent::GamepadButtonPressed(_, _) | InputEvent::GamepadButtonReleased(_, _) | InputEvent::GamepadJoystickMoved(_, _, _) => {
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
                InputEvent::KeyPressed(Key::ArrowUp) |
                InputEvent::KeyPressed(Key::W) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::DPadUp)
                => {
                    Some(UiInputEvent::Up)
                }
                InputEvent::KeyPressed(Key::ArrowDown) |
                InputEvent::KeyPressed(Key::S) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::DPadDown)
                => {
                    Some(UiInputEvent::Down)
                }
                InputEvent::KeyPressed(Key::ArrowLeft) |
                InputEvent::KeyPressed(Key::A) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::DPadLeft)
                => {
                    Some(UiInputEvent::Left)
                }
                InputEvent::KeyPressed(Key::ArrowRight) |
                InputEvent::KeyPressed(Key::D) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::DPadRight)
                => {
                    Some(UiInputEvent::Right)
                }
                InputEvent::KeyPressed(Key::Enter) |
                InputEvent::KeyPressed(Key::Space) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::Start) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::South)
                => {
                    Some(UiInputEvent::SelectPressed)
                }
                InputEvent::KeyReleased(Key::Enter) |
                InputEvent::KeyReleased(Key::Space) |
                InputEvent::GamepadButtonReleased(_, GamepadButtonType::Start) |
                InputEvent::GamepadButtonReleased(_, GamepadButtonType::South)
                => {
                    Some(UiInputEvent::SelectReleased)
                }
                InputEvent::KeyPressed(Key::Escape) |
                InputEvent::KeyPressed(Key::Backspace) |
                InputEvent::GamepadButtonPressed(_, GamepadButtonType::East)
                => {
                    Some(UiInputEvent::Back)
                }
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
    Back
}

pub fn ui_receive_input(ui: &mut Ui, input: UiInput) {
    match input {
        UiInput::Mouse(x, y, pressed) => {
            ui.clear_hover();
            ui_update_hover(ui, &Ui::ROOT_NODE_ID, x, y, (0.0, 0.0));

            let current_pressed = ui.get_select_pressed();
            if pressed {
                if !current_pressed {
                    ui.set_select_pressed(true);
                    if let Some(hover_node) = ui.get_hover() {
                        if ui.node_ref(&hover_node).unwrap().widget_kind() == WidgetKind::Button {
                            ui.emit_event(&hover_node, UiEvent::Clicked);
                        }
                    }
                }
            } else {
                if current_pressed {
                    ui.set_select_pressed(false);
                }
            }
        }
        UiInput::Events(events) => {
            let hover_node = ui.get_hover();
            if hover_node.is_none() {
                let Some(first_input_id) = ui.get_first_input() else {
                    panic!("no first input set, cannot process input events without somewhere to start");
                };
                ui.receive_hover(&first_input_id);
                return;
            }
            let hover_node = hover_node.unwrap();
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
                        if let Some(next_id) = ui.nav_get_left_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::Right => {
                        if let Some(next_id) = ui.nav_get_right_id(&hover_node) {
                            ui.receive_hover(&next_id);
                        }
                    }
                    UiInputEvent::SelectPressed => {
                        ui.emit_event(&hover_node, UiEvent::Clicked);
                        ui.set_select_pressed(true);
                    }
                    UiInputEvent::SelectReleased => {
                        ui.set_select_pressed(false);
                    }
                    UiInputEvent::Back => {

                    }
                }
            }
        }
    }
}

// this currently requires recursion because node layout is additive ... one day we should fix this
fn ui_update_hover(
    ui: &mut Ui,
    id: &NodeId,
    mouse_x: f32,
    mouse_y: f32,
    parent_position: (f32, f32),
) {
    let Some(node) = ui.store.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    if !node.visible {
        return;
    }

    let Some((width, height, child_offset_x, child_offset_y)) = ui.cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let child_position = (
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
    );

    match node.widget_kind() {
        WidgetKind::Panel => {
            let Some(panel_ref) = ui.store.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                ui_update_hover(ui, &child_id, mouse_x, mouse_y, child_position);
            }
        }
        WidgetKind::Button => {
            let Some(button_mut) = ui.store.button_mut(id) else {
                panic!("no button mut for node_id: {:?}", id);
            };
            if button_mut.mouse_is_inside(
                (width, height, child_position.0, child_position.1),
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
                (width, height, child_position.0, child_position.1),
                mouse_x, mouse_y,
            ) {
                ui.receive_hover(id);
                ui.set_cursor_icon(CursorIcon::Text);
            }
        }
        _ => {}
    }
}