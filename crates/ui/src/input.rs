use bevy_ecs::event::EventReader;
use bevy_log::warn;

use input::{CursorIcon, GamepadButtonType, Input, InputEvent, Key, Modifiers, MouseButton};
use math::Vec2;
use ui_layout::TextMeasurer;

use crate::{NodeId, Ui, UiNodeEvent, WidgetKind};

pub struct UiInputConverter;

impl UiInputConverter {

    pub fn convert(input_events: &mut EventReader<InputEvent>) -> Option<UiInput> {
        let mut output_mode = None;
        let mut keyboard_events = None;
        let mut mouse_events = None;
        let mut mouse_position = None;

        for input_event in input_events.read() {

            // first, get the mode of the input event
            if let Some(was_mouse) = match input_event {
                InputEvent::MouseClicked(_, _, _) | InputEvent::MouseDoubleClicked(_, _, _) | InputEvent::MouseTripleClicked(_, _, _) |
                InputEvent::MouseReleased(_) | InputEvent::MouseMoved(_) |
                InputEvent::MouseDragged(_, _, _, _) | InputEvent::MouseMiddleScrolled(_) => {
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

            let Some(was_mouse) = output_mode else {
                continue;
            };
            if was_mouse {
                // collect the keyboard events
                let output_event = match input_event {
                    InputEvent::MouseMoved(position) => {
                        mouse_position = Some(*position);
                        Some(MouseEvent::Move)
                    },
                    InputEvent::MouseDragged(button, position, _delta, modifiers) => {
                        mouse_position = Some(*position);
                        Some(MouseEvent::Drag(*button, *modifiers))
                    }
                    InputEvent::MouseClicked(button, position, modifiers) => {
                        mouse_position = Some(*position);
                        Some(MouseEvent::SingleClick(*button, *position, *modifiers))
                    }
                    InputEvent::MouseDoubleClicked(button, position, _modifiers) => {
                        mouse_position = Some(*position);
                        Some(MouseEvent::DoubleClick(*button, *position))
                    }
                    InputEvent::MouseTripleClicked(button, position, _modifiers) => {
                        mouse_position = Some(*position);
                        Some(MouseEvent::TripleClick(*button, *position))
                    }
                    InputEvent::MouseReleased(button) => {
                        Some(MouseEvent::Release(*button))
                    }
                    _ => None,
                };
                let Some(mouse_event) = output_event else {
                    continue;
                };

                if mouse_events.is_none() {
                    mouse_events = Some(Vec::new());
                }
                mouse_events.as_mut().unwrap().push(mouse_event);
            } else {
                // collect the keyboard events
                let output_event = match input_event {
                    InputEvent::GamepadButtonPressed(_, button) => {
                        match button {
                            GamepadButtonType::DPadUp => Some(KeyboardOrGamepadEvent::Up),
                            GamepadButtonType::DPadDown => Some(KeyboardOrGamepadEvent::Down),
                            GamepadButtonType::DPadLeft => Some(KeyboardOrGamepadEvent::Left(Modifiers::default())),
                            GamepadButtonType::DPadRight => Some(KeyboardOrGamepadEvent::Right(Modifiers::default())),
                            GamepadButtonType::Start | GamepadButtonType::South => Some(KeyboardOrGamepadEvent::SelectPressed),
                            GamepadButtonType::East => Some(KeyboardOrGamepadEvent::Back),
                            _ => None,
                        }
                    }
                    InputEvent::GamepadButtonReleased(_, button) => {
                        match button {
                            GamepadButtonType::Start | GamepadButtonType::South => Some(KeyboardOrGamepadEvent::SelectReleased),
                            _ => None,
                        }
                    }
                    InputEvent::KeyPressed(key, modifiers) => {
                        match key {
                            Key::ArrowUp => Some(KeyboardOrGamepadEvent::Up),
                            Key::ArrowDown => Some(KeyboardOrGamepadEvent::Down),
                            Key::ArrowLeft => Some(KeyboardOrGamepadEvent::Left(*modifiers)),
                            Key::ArrowRight => Some(KeyboardOrGamepadEvent::Right(*modifiers)),
                            Key::Enter => Some(KeyboardOrGamepadEvent::SelectPressed),
                            Key::Escape => Some(KeyboardOrGamepadEvent::Back),
                            Key::Backspace => Some(KeyboardOrGamepadEvent::Backspace(*modifiers)),
                            Key::Delete => Some(KeyboardOrGamepadEvent::Delete(*modifiers)),
                            Key::Home => Some(KeyboardOrGamepadEvent::Home(*modifiers)),
                            Key::End => Some(KeyboardOrGamepadEvent::End(*modifiers)),
                            Key::A => {
                                if modifiers.ctrl {
                                    Some(KeyboardOrGamepadEvent::SelectAll)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        }
                    }
                    InputEvent::KeyReleased(key) => {
                        match key {
                            Key::Enter => Some(KeyboardOrGamepadEvent::SelectReleased),
                            _ => None,
                        }
                    }
                    InputEvent::Text(c) => Some(KeyboardOrGamepadEvent::Text(*c)),
                    InputEvent::Paste(text) => Some(KeyboardOrGamepadEvent::Paste(text.clone())),
                    InputEvent::Copy => Some(KeyboardOrGamepadEvent::Copy),
                    InputEvent::Cut => Some(KeyboardOrGamepadEvent::Cut),
                    _ => None,
                };
                let Some(keyboard_event) = output_event else {
                    continue;
                };

                if keyboard_events.is_none() {
                    keyboard_events = Some(Vec::new());
                }
                keyboard_events.as_mut().unwrap().push(keyboard_event);
            }
        }

        let Some(last_was_mouse) = output_mode else {
            return None;
        };
        if last_was_mouse {
            if let Some(output_events) = mouse_events {
                return Some(UiInput::Mouse(mouse_position, output_events));
            } else {
                return None;
            }
        } else {
            if let Some(output_events) = keyboard_events {
                return Some(UiInput::KeyboardOrGamepad(output_events));
            } else {
                return None;
            }
        }
    }
}

#[derive(Clone)]
pub enum MouseEvent {
    // release
    Release(MouseButton),
    // button, click position, modifiers
    SingleClick(MouseButton, Vec2, Modifiers),
    // button, click position
    DoubleClick(MouseButton, Vec2),
    // button, click position
    TripleClick(MouseButton, Vec2),
    // button
    Drag(MouseButton, Modifiers),
    // position
    Move,
}

#[derive(Clone)]
pub enum UiInput {
    Mouse(Option<Vec2>, Vec<MouseEvent>),
    KeyboardOrGamepad(Vec<KeyboardOrGamepadEvent>)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum KeyboardOrGamepadEvent {
    Up, Down, Left(Modifiers), Right(Modifiers),
    SelectPressed, SelectReleased,
    Back, Backspace(Modifiers), Delete(Modifiers), Text(char), Home(Modifiers), End(Modifiers),
    Copy, Cut, Paste(String), SelectAll,
}

pub fn ui_receive_input(ui: &mut Ui, text_measurer: &dyn TextMeasurer, input: UiInput) {
    match input {
        UiInput::Mouse(mouse_position, events) => {

            if let Some(mouse_position) = mouse_position {
                ui.clear_hover();

                for node_id in 0..ui.store.nodes.len() {
                    let node_id = NodeId::from_usize(node_id);
                    ui_update_hover(ui, &node_id, mouse_position.x, mouse_position.y);
                }
            }

            let hover_node = ui.get_hover().map(|id| {
                (id, ui.node_ref(&id).unwrap().widget_kind())
            });
            let active_node = ui.get_active_node().map(|id| {
                (id, ui.node_ref(&id).unwrap().widget_kind())
            });
            for event in events {
                match event {
                    MouseEvent::Release(MouseButton::Left) => {
                        if let Some((_, WidgetKind::Button)) = active_node {
                            ui.set_active_node(None);
                        }
                    },
                    MouseEvent::SingleClick(MouseButton::Left, _, _) | MouseEvent::DoubleClick(MouseButton::Left, _) | MouseEvent::TripleClick(MouseButton::Left, _) => {
                        if let Some((hover_node, kind)) = hover_node {
                            match kind {
                                WidgetKind::Button => {
                                    ui.set_active_node(Some(hover_node));
                                    ui.emit_node_event(&hover_node, UiNodeEvent::Clicked);
                                }
                                WidgetKind::Textbox => {
                                    ui.set_active_node(Some(hover_node));
                                    ui.reset_interact_timer();
                                    let (_, node_height, node_x, _, _) = ui.cache.bounds(&hover_node).unwrap();
                                    ui.textbox_mut(&hover_node).unwrap().recv_mouse_event(text_measurer, node_x, node_height, mouse_position, event);
                                }
                                _ => {}
                            }
                        }
                    }
                    MouseEvent::Drag(MouseButton::Left, _) => {
                        if let Some((hover_node, WidgetKind::Textbox)) = hover_node {
                            ui.reset_interact_timer();
                            let (_, node_height, node_x, _, _) = ui.cache.bounds(&hover_node).unwrap();
                            ui.textbox_mut(&hover_node).unwrap().recv_mouse_event(text_measurer, node_x, node_height, mouse_position, event);
                        }
                    }
                    MouseEvent::Move => {}
                    _ => {}
                }
            }
        }
        UiInput::KeyboardOrGamepad(mut events) => {
            let mut hover_node = ui.get_hover();
            let mut active_node = ui.get_active_node().map(|id| {
                (id, ui.node_ref(&id).unwrap().widget_kind())
            });

            // textbox events
            {
                if let Some((textbox_id, WidgetKind::Textbox)) = active_node {
                    let mut next_events = Vec::new();
                    for input_event in events {
                        match input_event {
                            KeyboardOrGamepadEvent::Right(_) | KeyboardOrGamepadEvent::Left(_) | KeyboardOrGamepadEvent::Backspace(_) | KeyboardOrGamepadEvent::Delete(_) |
                            KeyboardOrGamepadEvent::Text(_) | KeyboardOrGamepadEvent::Home(_) | KeyboardOrGamepadEvent::End(_) | KeyboardOrGamepadEvent::Paste(_) |
                            KeyboardOrGamepadEvent::Copy | KeyboardOrGamepadEvent::Cut | KeyboardOrGamepadEvent::SelectAll
                            => {
                                ui.reset_interact_timer();
                                if let Some(output_events) = ui.textbox_mut(&textbox_id).unwrap().recv_keyboard_or_gamepad_event(input_event) {
                                    for output_event in output_events {
                                        ui.emit_global_event(output_event);
                                    }
                                }
                            }
                            _ => {
                                next_events.push(input_event);
                            }
                        }
                    }
                    events = next_events;
                }
            }

            // handle navigation of hover elements & button activation
            for event in &events {
                match event {
                    KeyboardOrGamepadEvent::Up | KeyboardOrGamepadEvent::Down | KeyboardOrGamepadEvent::Left(_) | KeyboardOrGamepadEvent::Right(_) => {

                        // navigation ...

                        // make sure hovering
                        // hover default ui element if none is hovered (coming from mouse mode)
                        if hover_node.is_none() {
                            let Some(first_input_id) = ui.get_first_input() else {
                                panic!("no first input set, cannot process input events without somewhere to start");
                            };
                            ui.receive_hover(&first_input_id);
                            hover_node = Some(first_input_id);
                            continue;
                        }

                        // handle navigation
                        let hover_node_inside = hover_node.unwrap();
                        if let Some(next_id) = match event {
                            KeyboardOrGamepadEvent::Up => ui.nav_get_up_id(&hover_node_inside),
                            KeyboardOrGamepadEvent::Down => ui.nav_get_down_id(&hover_node_inside),
                            KeyboardOrGamepadEvent::Left(_) => ui.nav_get_left_id(&hover_node_inside),
                            KeyboardOrGamepadEvent::Right(_) => ui.nav_get_right_id(&hover_node_inside),
                            _ => None,
                        } {
                            ui.receive_hover(&next_id);
                            hover_node = Some(next_id);

                            // deselect any other active nodes, as we've navigated away from it
                            if active_node.is_some() {
                                ui.set_active_node(None);
                                active_node = None;
                            }
                        }
                    }
                    KeyboardOrGamepadEvent::SelectPressed => {
                        // if hover node is already hovering, can handle select pressed
                        if let Some(hover_node) = hover_node {
                            let widget_kind = ui.node_ref(&hover_node).unwrap().widget_kind();
                            match widget_kind {
                                WidgetKind::Button => {
                                    ui.set_active_node(Some(hover_node));
                                    ui.emit_node_event(&hover_node, UiNodeEvent::Clicked);
                                }
                                WidgetKind::Textbox => {
                                    ui.set_active_node(Some(hover_node));
                                    ui.textbox_mut(&hover_node).unwrap().recv_keyboard_or_gamepad_event(KeyboardOrGamepadEvent::End(Modifiers::default()));
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyboardOrGamepadEvent::SelectReleased => {
                        if let Some(active_node) = ui.get_active_node() {
                            match ui.node_ref(&active_node).unwrap().widget_kind() {
                                WidgetKind::Button => {
                                    ui.set_active_node(None);
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