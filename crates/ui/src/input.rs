use bevy_ecs::event::EventReader;
use bevy_log::{info, warn};

use input::{GamepadButtonType, Input, InputEvent, Key, MouseButton};

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
                    Some(UiInputEvent::Select)
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
    Up, Down, Left, Right, Select, Back
}

pub fn ui_receive_input(ui: &mut Ui, input: UiInput) {
    match input {
        UiInput::Mouse(x, y, pressed) => {
            let mouse_state = (x, y, pressed);
            update_button_states(ui, &Ui::ROOT_NODE_ID, mouse_state, (0.0, 0.0));
        }
        UiInput::Events(events) => {
            info!("processing ui input events...");
        }
    }
}

fn update_button_states(
    ui: &mut Ui,
    id: &NodeId,
    mouse_state: (f32, f32, bool),
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
                update_button_states(ui, &child_id, mouse_state, child_position);
            }
        }
        WidgetKind::Button => {
            let Some(button_mut) = ui.store.button_mut(id) else {
                panic!("no button mut for node_id: {:?}", id);
            };
            let did_click = button_mut.update_state(
                (width, height, child_position.0, child_position.1),
                mouse_state,
            );
            if did_click {
                ui.emit_event(id, UiEvent::Clicked);
            }
        }
        _ => {}
    }
}