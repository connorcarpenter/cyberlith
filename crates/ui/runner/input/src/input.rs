use bevy_ecs::event::EventReader;
use bevy_log::warn;

use input::{CursorIcon, GamepadButtonType, InputEvent, Key, Modifiers, MouseButton};
use math::Vec2;

use ui_runner_config::{TextMeasurer, NodeId, UiRuntimeConfig, WidgetKind, point_is_inside};
use ui_state::UiState;

use crate::{UiGlobalEvent, UiNodeEvent, input_state::UiInputState, textbox_input_state::TextboxInputState};

pub struct UiInputConverter;

impl UiInputConverter {

    pub fn convert(input_events: &mut EventReader<InputEvent>) -> Option<(Option<Vec2>, Vec<UiInputEvent>)> {
        let mut mouse_position = None;
        let mut output_events = None;

        for input_event in input_events.read() {

            let output_event = match input_event {

                // Mouse
                InputEvent::MouseMoved(position) => {
                    mouse_position = Some(*position);
                    Some(UiInputEvent::MouseMove)
                },
                InputEvent::MouseDragged(button, position, _delta, modifiers) => {
                    mouse_position = Some(*position);
                    Some(UiInputEvent::MouseButtonDrag(*button, *modifiers))
                }
                InputEvent::MouseClicked(button, position, modifiers) => {
                    mouse_position = Some(*position);
                    Some(UiInputEvent::MouseSingleClick(*button, *position, *modifiers))
                }
                InputEvent::MouseDoubleClicked(button, position, _modifiers) => {
                    mouse_position = Some(*position);
                    Some(UiInputEvent::MouseDoubleClick(*button, *position))
                }
                InputEvent::MouseTripleClicked(button, position, _modifiers) => {
                    mouse_position = Some(*position);
                    Some(UiInputEvent::MouseTripleClick(*button, *position))
                }
                InputEvent::MouseReleased(button) => {
                    Some(UiInputEvent::MouseButtonRelease(*button))
                }
                // Gamepad
                InputEvent::GamepadButtonPressed(_, button) => {
                    match button {
                        GamepadButtonType::DPadUp => Some(UiInputEvent::UpPressed),
                        GamepadButtonType::DPadDown => Some(UiInputEvent::DownPressed),
                        GamepadButtonType::DPadLeft => Some(UiInputEvent::LeftPressed(Modifiers::default())),
                        GamepadButtonType::DPadRight => Some(UiInputEvent::RightPressed(Modifiers::default())),
                        GamepadButtonType::Start | GamepadButtonType::South => Some(UiInputEvent::SelectPressed),
                        GamepadButtonType::East => Some(UiInputEvent::BackPressed),
                        _ => None,
                    }
                }
                InputEvent::GamepadButtonReleased(_, button) => {
                    match button {
                        GamepadButtonType::Start | GamepadButtonType::South => Some(UiInputEvent::SelectReleased),
                        GamepadButtonType::DPadLeft => Some(UiInputEvent::LeftReleased),
                        GamepadButtonType::DPadRight => Some(UiInputEvent::RightReleased),
                        _ => None,
                    }
                }
                InputEvent::KeyPressed(key, modifiers) => {
                    match key {
                        Key::ArrowUp => Some(UiInputEvent::UpPressed),
                        Key::ArrowDown => Some(UiInputEvent::DownPressed),
                        Key::ArrowLeft => Some(UiInputEvent::LeftPressed(*modifiers)),
                        Key::ArrowRight => Some(UiInputEvent::RightPressed(*modifiers)),
                        Key::Enter => Some(UiInputEvent::SelectPressed),
                        Key::Tab => Some(UiInputEvent::TabPressed),
                        Key::Escape => Some(UiInputEvent::BackPressed),
                        Key::Backspace => Some(UiInputEvent::BackspacePressed(*modifiers)),
                        Key::Delete => Some(UiInputEvent::DeletePressed(*modifiers)),
                        Key::Home => Some(UiInputEvent::HomePressed(*modifiers)),
                        Key::End => Some(UiInputEvent::EndPressed(*modifiers)),
                        Key::A => {
                            if modifiers.ctrl {
                                Some(UiInputEvent::TextSelectAll)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
                InputEvent::KeyReleased(key) => {
                    match key {
                        Key::Enter => Some(UiInputEvent::SelectReleased),
                        Key::ArrowLeft => Some(UiInputEvent::LeftReleased),
                        Key::ArrowRight => Some(UiInputEvent::RightReleased),
                        _ => None,
                    }
                }
                InputEvent::Text(c) => Some(UiInputEvent::TextInsert(*c)),
                InputEvent::Paste(text) => Some(UiInputEvent::TextPaste(text.clone())),
                InputEvent::Copy => Some(UiInputEvent::TextCopy),
                InputEvent::Cut => Some(UiInputEvent::TextCut),
                _ => None,
            };

            let Some(output_event) = output_event else {
                continue;
            };

            if output_events.is_none() {
                output_events = Some(Vec::new());
            }
            output_events.as_mut().unwrap().push(output_event);
        }

        if let Some(output_events) = output_events {
            Some((mouse_position, output_events))
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum UiInputEvent {

    // keyboard/gamepad
    UpPressed,
    DownPressed,
    LeftPressed(Modifiers),
    RightPressed(Modifiers),
    LeftReleased,
    RightReleased,
    TabPressed,
    SelectPressed, SelectReleased,
    BackPressed,
    BackspacePressed(Modifiers),
    DeletePressed(Modifiers),
    TextInsert(char),
    HomePressed(Modifiers),
    EndPressed(Modifiers),
    TextCopy,
    TextCut,
    TextPaste(String),
    TextSelectAll,

    // mouse
    MouseButtonRelease(MouseButton),
    // button, click position, modifiers
    MouseSingleClick(MouseButton, Vec2, Modifiers),
    // button, click position
    MouseDoubleClick(MouseButton, Vec2),
    // button, click position
    MouseTripleClick(MouseButton, Vec2),
    // button
    MouseButtonDrag(MouseButton, Modifiers),
    // position
    MouseMove,
}

impl UiInputEvent {
    pub fn is_mouse_event(&self) -> bool {
        match self {
            Self::MouseButtonRelease(_) | Self::MouseSingleClick(_, _, _) |
            Self::MouseDoubleClick(_, _) | Self::MouseTripleClick(_, _) | Self::MouseButtonDrag(_, _) | Self::MouseMove => {
                true
            }
            _ => false,
        }
    }
}

pub fn ui_receive_input(
    ui_config: &UiRuntimeConfig,
    ui_state: &mut UiState,
    ui_input_state: &mut UiInputState,
    text_measurer: &dyn TextMeasurer,
    mouse_position: Option<Vec2>,
    events: Vec<UiInputEvent>
) {

    let mut mouse_event_has_ocurred = false;
    let mut mouse_hover_node = None;
    let mut mouse_active_node = None;

    let mut kb_or_gp_events = Vec::new();

    for event in events {
        if event.is_mouse_event() {
            if !mouse_event_has_ocurred {
                mouse_event_has_ocurred = true;

                if let Some(mouse_position) = mouse_position {
                    ui_input_state.set_cursor_icon(CursorIcon::Default);
                    ui_input_state.clear_hover();

                    for node_id in 0..ui_state.store.nodes.len() {
                        let node_id = NodeId::from_usize(node_id);
                        ui_update_hover(ui_config, ui_state, ui_input_state, &node_id, mouse_position.x, mouse_position.y);
                    }
                }

                mouse_hover_node = ui_input_state.get_hover().map(|id| {
                    (id, ui_config.get_node(&id).unwrap().widget_kind())
                });
                mouse_active_node = ui_input_state.get_active_node().map(|id| {
                    (id, ui_config.get_node(&id).unwrap().widget_kind())
                });
            }

            match event {
                UiInputEvent::MouseButtonRelease(MouseButton::Left) => {
                    if let Some((_, WidgetKind::Button)) = mouse_active_node {
                        ui_input_state.set_active_node(None);
                    }
                },
                UiInputEvent::MouseSingleClick(MouseButton::Left, _, _) | UiInputEvent::MouseDoubleClick(MouseButton::Left, _) | UiInputEvent::MouseTripleClick(MouseButton::Left, _) => {
                    if let Some((hover_node, kind)) = mouse_hover_node {
                        match kind {
                            WidgetKind::Button => {
                                ui_input_state.set_active_node(Some(hover_node));
                                ui_input_state.emit_node_event(&hover_node, UiNodeEvent::Clicked);
                            }
                            WidgetKind::Textbox => {
                                ui_input_state.set_active_node(Some(hover_node));
                                ui_input_state.reset_interact_timer();
                                let (_, node_height, node_x, _, _) = ui_state.cache.bounds(&hover_node).unwrap();
                                let textbox_state = ui_state.store.textbox_mut(&hover_node).unwrap();
                                TextboxInputState::recv_mouse_event(ui_input_state, text_measurer, textbox_state, node_x, node_height, mouse_position, event);
                            }
                            _ => {}
                        }
                    }
                }
                UiInputEvent::MouseButtonDrag(MouseButton::Left, _) => {
                    if let Some((hover_node, WidgetKind::Textbox)) = mouse_hover_node {
                        ui_input_state.reset_interact_timer();
                        let (_, node_height, node_x, _, _) = ui_state.cache.bounds(&hover_node).unwrap();
                        let textbox_state = ui_state.store.textbox_mut(&hover_node).unwrap();
                        TextboxInputState::recv_mouse_event(ui_input_state, text_measurer, textbox_state, node_x, node_height, mouse_position, event);
                    }
                }
                UiInputEvent::MouseMove => {}
                _ => {}
            }
        } else {
            kb_or_gp_events.push(event);
        }
    }

    if kb_or_gp_events.is_empty() {
        return;
    }

    let mut hover_node = ui_input_state.get_hover();
    let mut active_node = ui_input_state.get_active_node().map(|id| {
        (id, ui_config.get_node(&id).unwrap().widget_kind())
    });
    let mut events = kb_or_gp_events;

    // textbox events
    if let Some((textbox_id, WidgetKind::Textbox)) = active_node {
        let mut next_events = Vec::new();
        for input_event in events {
            match &input_event {
                UiInputEvent::RightPressed(_) | UiInputEvent::LeftPressed(_) | UiInputEvent::BackspacePressed(_) | UiInputEvent::DeletePressed(_) |
                UiInputEvent::TextInsert(_) | UiInputEvent::HomePressed(_) | UiInputEvent::EndPressed(_) | UiInputEvent::TextPaste(_) |
                UiInputEvent::TextCopy | UiInputEvent::TextCut | UiInputEvent::TextSelectAll
                => {
                    ui_input_state.reset_interact_timer();

                    let textbox_state = ui_state.textbox_mut(&textbox_id).unwrap();
                    let output_events = TextboxInputState::recv_keyboard_or_gamepad_event(ui_input_state, textbox_state, input_event.clone());

                    if let Some(output_events) = output_events {
                        for output_event in output_events {
                            match &output_event {
                                UiGlobalEvent::Copied(_) => {
                                    ui_input_state.emit_global_event(output_event);
                                }
                                UiGlobalEvent::PassThru => {
                                    next_events.push(input_event.clone());
                                }
                            }
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

    // handle navigation of hover elements & button activation
    for event in &events {
        match event {
            UiInputEvent::UpPressed | UiInputEvent::DownPressed | UiInputEvent::LeftPressed(_) | UiInputEvent::RightPressed(_) | UiInputEvent::TabPressed => {

                // navigation ...
                if let Some((active_id, WidgetKind::Textbox)) = active_node {
                    ui_input_state.receive_hover(&active_id);
                    hover_node = Some(active_id);
                    ui_input_state.set_active_node(None);
                    active_node = None;
                }
                // make sure hovering
                // hover default ui element if none is hovered (coming from mouse mode)
                if hover_node.is_none() {
                    let first_input_id = ui_config.get_first_input();
                    ui_input_state.receive_hover(&first_input_id);
                    hover_node = Some(first_input_id);
                    continue;
                }

                // handle navigation
                let hover_node_inside = hover_node.unwrap();
                if let Some(next_id) = match event {
                    UiInputEvent::UpPressed => ui_config.nav_get_up_id(&hover_node_inside),
                    UiInputEvent::DownPressed => ui_config.nav_get_down_id(&hover_node_inside),
                    UiInputEvent::LeftPressed(_) => ui_config.nav_get_left_id(&hover_node_inside),
                    UiInputEvent::RightPressed(_) => ui_config.nav_get_right_id(&hover_node_inside),
                    UiInputEvent::TabPressed => ui_config.nav_get_tab_id(&hover_node_inside),
                    _ => None,
                } {
                    ui_input_state.receive_hover(&next_id);
                    hover_node = Some(next_id);
                }
            }
            UiInputEvent::BackPressed => {
                if let Some((id, WidgetKind::Textbox)) = active_node {
                    // make textbox inactive
                    ui_input_state.set_active_node(None);
                    active_node = None;
                    // hover textbox
                    ui_input_state.receive_hover(&id);
                    hover_node = Some(id);
                } else {
                    // de-hover
                    if hover_node.is_some() {
                        ui_input_state.clear_hover();
                        hover_node = None;
                    }
                }
            }
            UiInputEvent::SelectPressed => {
                if let Some((active_id, WidgetKind::Textbox)) = active_node {
                    if let Some(next_id) = ui_config.nav_get_tab_id(&active_id) {
                        match ui_config.get_node(&next_id).unwrap().widget_kind() {
                            WidgetKind::Textbox => {
                                // make next textbox active
                                ui_input_state.set_active_node(Some(next_id));
                                active_node = None;
                                // clear hover
                                ui_input_state.clear_hover();
                                hover_node = None;
                            }
                            WidgetKind::Button => {
                                // make textbox inactive
                                ui_input_state.set_active_node(None);
                                active_node = None;
                                // hover button
                                ui_input_state.receive_hover(&next_id);
                                hover_node = Some(next_id);
                            }
                            _ => panic!("no navigation for other types")
                        }

                    } else {
                        // make textbox inactive
                        ui_input_state.set_active_node(None);
                        active_node = None;
                        // hover textbox
                        ui_input_state.receive_hover(&active_id);
                        hover_node = Some(active_id);
                    }
                }
                // if hover node is already hovering, can handle select pressed
                else if let Some(hover_id) = hover_node {
                    let widget_kind = ui_config.get_node(&hover_id).unwrap().widget_kind();
                    match widget_kind {
                        WidgetKind::Button => {
                            ui_input_state.set_active_node(Some(hover_id));
                            ui_input_state.emit_node_event(&hover_id, UiNodeEvent::Clicked);
                        }
                        WidgetKind::Textbox => {
                            ui_input_state.set_active_node(Some(hover_id));
                            let textbox_state = ui_state.textbox_mut(&hover_id).unwrap();
                            TextboxInputState::recv_keyboard_or_gamepad_event(ui_input_state, textbox_state, UiInputEvent::EndPressed(Modifiers::default()));
                        }
                        _ => {}
                    }
                }
            }
            UiInputEvent::SelectReleased => {
                if let Some(active_node) = ui_input_state.get_active_node() {
                    match ui_config.get_node(&active_node).unwrap().widget_kind() {
                        WidgetKind::Button => {
                            ui_input_state.set_active_node(None);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

fn ui_update_hover(
    ui_config: &UiRuntimeConfig,
    ui_state: &mut UiState,
    ui_input_state: &mut UiInputState,
    id: &NodeId,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some(visible) = ui_state.visibility_store.get_node_visibility(id) else {
        warn!("no node for id: {:?}", id);
        return;
    };
    if !visible {
        return;
    }
    let Some((width, height, child_offset_x, child_offset_y, _)) = ui_state.cache.bounds(id) else {
        warn!("no bounds for id 2: {:?}", id);
        return;
    };

    let Some(node) = ui_config.get_node(&id) else {
        warn!("no node for id: {:?}", id);
        return;
    };
    let check = match node.widget_kind() {
        WidgetKind::Button => Some(CursorIcon::Hand),
        WidgetKind::Textbox => Some(CursorIcon::Text),
        _ => None,
    };
    if let Some(cursor_icon) = check {
        if point_is_inside(
            (width, height, child_offset_x, child_offset_y),
            mouse_x, mouse_y,
        ) {
            ui_input_state.receive_hover(id);
            ui_input_state.set_cursor_icon(cursor_icon);
        }
    }
}