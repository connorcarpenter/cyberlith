use asset_id::AssetId;
use logging::warn;
use input::{CursorIcon, GamepadButtonType, InputEvent, Key, Modifiers, MouseButton};
use math::Vec2;
use ui_runner_config::{point_is_inside, NodeId, TextMeasurer, UiRuntimeConfig, WidgetKind};
use ui_state::UiState;

use crate::{
    input_state::UiInputState, textbox_input_state::TextboxInputState, UiGlobalEvent, UiNodeEvent,
};

pub struct UiInputConverter;

impl UiInputConverter {
    pub fn convert(next_inputs: Vec<InputEvent>) -> Option<(Option<Vec2>, Vec<UiInputEvent>)> {
        let mut mouse_position = None;
        let mut output_events = None;

        for input_event in next_inputs {
            let output_event = match input_event {
                // Mouse
                InputEvent::MouseMoved(position) => {
                    mouse_position = Some(position);
                    Some(UiInputEvent::MouseMove)
                }
                InputEvent::MouseDragged(button, position, _delta, modifiers) => {
                    mouse_position = Some(position);
                    Some(UiInputEvent::MouseButtonDrag(button, modifiers))
                }
                InputEvent::MouseClicked(button, position, modifiers) => {
                    mouse_position = Some(position);
                    Some(UiInputEvent::MouseSingleClick(button, position, modifiers))
                }
                InputEvent::MouseDoubleClicked(button, position, _modifiers) => {
                    mouse_position = Some(position);
                    Some(UiInputEvent::MouseDoubleClick(button, position))
                }
                InputEvent::MouseTripleClicked(button, position, _modifiers) => {
                    mouse_position = Some(position);
                    Some(UiInputEvent::MouseTripleClick(button, position))
                }
                InputEvent::MouseReleased(button) => Some(UiInputEvent::MouseButtonRelease(button)),
                // Gamepad
                InputEvent::GamepadButtonPressed(_, button) => match button {
                    GamepadButtonType::DPadUp => Some(UiInputEvent::UpPressed),
                    GamepadButtonType::DPadDown => Some(UiInputEvent::DownPressed),
                    GamepadButtonType::DPadLeft => {
                        Some(UiInputEvent::LeftPressed(Modifiers::default()))
                    }
                    GamepadButtonType::DPadRight => {
                        Some(UiInputEvent::RightPressed(Modifiers::default()))
                    }
                    GamepadButtonType::Start | GamepadButtonType::South => {
                        Some(UiInputEvent::SelectPressed)
                    }
                    GamepadButtonType::East => Some(UiInputEvent::BackPressed),
                    _ => None,
                },
                InputEvent::GamepadButtonReleased(_, button) => match button {
                    GamepadButtonType::Start | GamepadButtonType::South => {
                        Some(UiInputEvent::SelectReleased)
                    }
                    GamepadButtonType::DPadLeft => Some(UiInputEvent::LeftReleased),
                    GamepadButtonType::DPadRight => Some(UiInputEvent::RightReleased),
                    _ => None,
                },
                InputEvent::KeyPressed(key, modifiers) => match key {
                    //
                    Key::ArrowUp => Some(UiInputEvent::UpPressed),
                    Key::ArrowDown => Some(UiInputEvent::DownPressed),
                    Key::ArrowLeft => Some(UiInputEvent::LeftPressed(modifiers)),
                    Key::ArrowRight => Some(UiInputEvent::RightPressed(modifiers)),
                    Key::Enter => Some(UiInputEvent::SelectPressed),
                    Key::Tab => Some(UiInputEvent::TabPressed),
                    Key::Escape => Some(UiInputEvent::BackPressed),
                    Key::Backspace => Some(UiInputEvent::BackspacePressed(modifiers)),
                    Key::Delete => Some(UiInputEvent::DeletePressed(modifiers)),
                    Key::Home => Some(UiInputEvent::HomePressed(modifiers)),
                    Key::End => Some(UiInputEvent::EndPressed(modifiers)),
                    Key::A => {
                        if modifiers.ctrl {
                            Some(UiInputEvent::TextSelectAll)
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                InputEvent::KeyReleased(key) => match key {
                    Key::Enter => Some(UiInputEvent::SelectReleased),
                    Key::ArrowLeft => Some(UiInputEvent::LeftReleased),
                    Key::ArrowRight => Some(UiInputEvent::RightReleased),
                    _ => None,
                },
                InputEvent::Text(c) => Some(UiInputEvent::CharacterInsert(c)),
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
    LeftHeld(Modifiers),
    RightPressed(Modifiers),
    RightHeld(Modifiers),
    LeftReleased,
    RightReleased,
    TabPressed,
    SelectPressed,
    SelectReleased,
    BackPressed,
    BackspacePressed(Modifiers),
    DeletePressed(Modifiers),
    CharacterInsert(char),
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
            Self::MouseButtonRelease(_)
            | Self::MouseSingleClick(_, _, _)
            | Self::MouseDoubleClick(_, _)
            | Self::MouseTripleClick(_, _)
            | Self::MouseButtonDrag(_, _)
            | Self::MouseMove => true,
            _ => false,
        }
    }
}

pub trait UiManagerTrait {
    fn ui_input_state(&self) -> &UiInputState;
    fn ui_input_state_mut(&mut self) -> &mut UiInputState;
    fn ui_state(&self, asset_id: &AssetId) -> &UiState;
    fn ui_state_mut(&mut self, asset_id: &AssetId) -> &mut UiState;
    fn root_ui_asset_id(&self) -> AssetId;
    fn nodes_len(&self, asset_id: &AssetId) -> usize;
    fn ui_config(&self, asset_id: &AssetId) -> Option<&UiRuntimeConfig>;
    fn textbox_receive_hover(&mut self, asset_id: &AssetId, node_id: &NodeId, bounds: (f32, f32, f32, f32), mouse_x: f32, mouse_y: f32) -> bool;
}

fn ui_receive_hover_recurse(
    ui_manager: &mut dyn UiManagerTrait,
    mouse_position: Vec2,
    ui_asset_id: &AssetId,
) {
    ui_manager.ui_input_state_mut().set_cursor_icon(CursorIcon::Default);
    ui_manager.ui_input_state_mut().clear_hover();

    for node_id in 0..ui_manager.nodes_len(ui_asset_id) {
        let node_id = NodeId::from_usize(node_id);
        ui_update_hover(
            ui_manager,
            ui_asset_id,
            &node_id,
            mouse_position.x,
            mouse_position.y,
        );
    }
}

pub fn ui_receive_input(
    ui_manager: &mut dyn UiManagerTrait,
    text_measurer: &dyn TextMeasurer,
    mouse_position: Option<Vec2>,
    events: Vec<UiInputEvent>,
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
                    let root_ui_node = ui_manager.root_ui_asset_id();

                    ui_receive_hover_recurse(ui_manager, mouse_position, &root_ui_node);
                }

                // add widget kind
                mouse_hover_node = ui_manager.ui_input_state()
                    .get_hover()
                    .map(|(asset_id, node_id)| {
                        let ui_config = ui_manager.ui_config(&asset_id).unwrap();
                        let widget_kind = ui_config.get_node(&node_id).unwrap().widget_kind();
                        (asset_id, node_id, widget_kind)
                    });
                mouse_active_node = ui_manager.ui_input_state()
                    .get_active_node()
                    .map(|(asset_id, node_id)| {
                        let ui_config = ui_manager.ui_config(&asset_id).unwrap();
                        let widget_kind = ui_config.get_node(&node_id).unwrap().widget_kind();
                        (asset_id, node_id, widget_kind)
                    });
            }

            match event {
                UiInputEvent::MouseButtonRelease(MouseButton::Left) => {
                    if let Some((_, _, WidgetKind::Button)) = mouse_active_node {
                        ui_manager.ui_input_state_mut().set_active_node(None);
                    }
                }
                UiInputEvent::MouseSingleClick(MouseButton::Left, _, _)
                | UiInputEvent::MouseDoubleClick(MouseButton::Left, _)
                | UiInputEvent::MouseTripleClick(MouseButton::Left, _) => {
                    if let Some((hover_asset_id, hover_node_id, kind)) = mouse_hover_node {
                        match kind {
                            WidgetKind::Button => {
                                ui_manager.ui_input_state_mut().set_active_node(Some((hover_asset_id, hover_node_id)));
                                ui_manager.ui_input_state_mut().emit_node_event(&hover_asset_id, &hover_node_id, UiNodeEvent::Clicked);
                            }
                            WidgetKind::Textbox => {
                                ui_manager.ui_input_state_mut().set_active_node(Some((hover_asset_id, hover_node_id)));
                                ui_manager.ui_input_state_mut().emit_node_event(&hover_asset_id, &hover_node_id, UiNodeEvent::Clicked);
                                ui_manager.ui_input_state_mut().reset_interact_timer();
                                let (_, node_height, node_x, _, _) =
                                    ui_manager.ui_state_mut(&hover_asset_id).cache.bounds(&hover_node_id).unwrap();
                                TextboxInputState::recv_mouse_event(
                                    ui_manager,
                                    text_measurer,
                                    node_x,
                                    node_height,
                                    mouse_position,
                                    &hover_asset_id,
                                    &hover_node_id,
                                    event,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                UiInputEvent::MouseButtonDrag(MouseButton::Left, _) => {
                    if let Some((hover_asset_id, hover_node_id, WidgetKind::Textbox)) = mouse_hover_node {
                        ui_manager.ui_input_state_mut().reset_interact_timer();
                        let (_, node_height, node_x, _, _) =
                            ui_manager.ui_state_mut(&hover_asset_id).cache.bounds(&hover_node_id).unwrap();
                        TextboxInputState::recv_mouse_event(
                            ui_manager,
                            text_measurer,
                            node_x,
                            node_height,
                            mouse_position,
                            &hover_asset_id,
                            &hover_node_id,
                            event,
                        );
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

    let mut hover_node = ui_manager.ui_input_state().get_hover();
    let mut active_node = ui_manager.ui_input_state()
        .get_active_node()
        .map(|(active_asset_id, active_node_id)| {
            let ui_config = ui_manager.ui_config(&active_asset_id).unwrap();
            let widget_kind = ui_config.get_node(&active_node_id).unwrap().widget_kind();
            (active_asset_id, active_node_id, widget_kind)
        });
    let mut events = kb_or_gp_events;

    // textbox events
    if let Some((ui_id, textbox_id, WidgetKind::Textbox)) = active_node {
        let mut next_events = Vec::new();
        for input_event in events {
            match &input_event {
                UiInputEvent::RightPressed(_)
                | UiInputEvent::RightHeld(_)
                | UiInputEvent::RightReleased
                | UiInputEvent::LeftPressed(_)
                | UiInputEvent::LeftHeld(_)
                | UiInputEvent::LeftReleased
                | UiInputEvent::BackspacePressed(_)
                | UiInputEvent::DeletePressed(_)
                | UiInputEvent::CharacterInsert(_)
                | UiInputEvent::HomePressed(_)
                | UiInputEvent::EndPressed(_)
                | UiInputEvent::TextPaste(_)
                | UiInputEvent::TextCopy
                | UiInputEvent::TextCut
                | UiInputEvent::TextSelectAll => {
                    ui_manager.ui_input_state_mut().reset_interact_timer();

                    let output_events = TextboxInputState::recv_keyboard_or_gamepad_event(
                        ui_manager,
                        text_measurer,
                        &ui_id,
                        &textbox_id,
                        input_event.clone(),
                    );

                    if let Some(output_events) = output_events {
                        for output_event in output_events {
                            match &output_event {
                                UiGlobalEvent::Copied(_) => {
                                    ui_manager.ui_input_state_mut().emit_global_event(output_event);
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
            UiInputEvent::UpPressed
            | UiInputEvent::DownPressed
            | UiInputEvent::LeftPressed(_)
            | UiInputEvent::RightPressed(_)
            | UiInputEvent::TabPressed => {
                // navigation ...
                if let Some((active_ui_id, active_node_id, WidgetKind::Textbox)) = active_node {
                    ui_manager.ui_input_state_mut().receive_hover(&active_ui_id, &active_node_id);
                    hover_node = Some((active_ui_id, active_node_id));
                    ui_manager.ui_input_state_mut().set_active_node(None);
                    active_node = None;
                }
                // make sure hovering
                // hover default ui element if none is hovered (coming from mouse mode)
                if hover_node.is_none() {
                    let root_ui_node = ui_manager.root_ui_asset_id();
                    let first_input_id = ui_manager.ui_config(&root_ui_node).unwrap().get_first_input();
                    ui_manager.ui_input_state_mut().receive_hover(&root_ui_node, &first_input_id);
                    hover_node = Some((root_ui_node, first_input_id));
                    continue;
                }

                // handle navigation
                let (hover_ui_inside, hover_node_inside) = hover_node.unwrap();
                if let Some(next_id) = match event {
                    UiInputEvent::UpPressed => ui_manager.ui_config(&hover_ui_inside).unwrap().nav_get_up_id(&hover_node_inside),
                    UiInputEvent::DownPressed => ui_manager.ui_config(&hover_ui_inside).unwrap().nav_get_down_id(&hover_node_inside),
                    UiInputEvent::LeftPressed(_) => ui_manager.ui_config(&hover_ui_inside).unwrap().nav_get_left_id(&hover_node_inside),
                    UiInputEvent::RightPressed(_) => ui_manager.ui_config(&hover_ui_inside).unwrap().nav_get_right_id(&hover_node_inside),
                    UiInputEvent::TabPressed => ui_manager.ui_config(&hover_ui_inside).unwrap().nav_get_tab_id(&hover_node_inside),
                    _ => None,
                } {
                    ui_manager.ui_input_state_mut().receive_hover(&hover_ui_inside, &next_id);
                    hover_node = Some((hover_ui_inside, next_id));
                }
            }
            UiInputEvent::BackPressed => {
                if let Some((asset_id, node_id, WidgetKind::Textbox)) = active_node {
                    // make textbox inactive
                    ui_manager.ui_input_state_mut().set_active_node(None);
                    active_node = None;
                    // hover textbox
                    ui_manager.ui_input_state_mut().receive_hover(&asset_id, &node_id);
                    hover_node = Some((asset_id, node_id));
                } else {
                    // de-hover
                    if hover_node.is_some() {
                        ui_manager.ui_input_state_mut().clear_hover();
                        hover_node = None;
                    }
                }
            }
            UiInputEvent::SelectPressed => {
                if let Some((active_ui_id, active_node_id, WidgetKind::Textbox)) = active_node {
                    if let Some(next_id) = ui_manager.ui_config(&active_ui_id).unwrap().nav_get_tab_id(&active_node_id) {
                        match ui_manager.ui_config(&active_ui_id).unwrap().get_node(&next_id).unwrap().widget_kind() {
                            WidgetKind::Textbox => {
                                // make next textbox active
                                ui_manager.ui_input_state_mut().set_active_node(Some((active_ui_id, next_id)));
                                active_node = None;
                                // clear hover
                                ui_manager.ui_input_state_mut().clear_hover();
                                hover_node = None;
                            }
                            WidgetKind::Button => {
                                // make textbox inactive
                                ui_manager.ui_input_state_mut().set_active_node(None);
                                active_node = None;
                                // hover button
                                ui_manager.ui_input_state_mut().receive_hover(&active_ui_id, &next_id);
                                hover_node = Some((active_ui_id, next_id));
                            }
                            _ => panic!("no navigation for other types"),
                        }
                    } else {
                        // make textbox inactive
                        ui_manager.ui_input_state_mut().set_active_node(None);
                        active_node = None;
                        // hover textbox
                        ui_manager.ui_input_state_mut().receive_hover(&active_ui_id, &active_node_id);
                        hover_node = Some((active_ui_id, active_node_id));
                    }
                }
                // if hover node is already hovering, can handle select pressed
                else if let Some((hover_ui_id, hover_node_id)) = hover_node {
                    let widget_kind = ui_manager.ui_config(&hover_ui_id).unwrap().get_node(&hover_node_id).unwrap().widget_kind();
                    match widget_kind {
                        WidgetKind::Button => {
                            ui_manager.ui_input_state_mut().set_active_node(Some((hover_ui_id, hover_node_id)));
                            ui_manager.ui_input_state_mut().emit_node_event(&hover_ui_id, &hover_node_id, UiNodeEvent::Clicked);
                        }
                        WidgetKind::Textbox => {
                            ui_manager.ui_input_state_mut().set_active_node(Some((hover_ui_id, hover_node_id)));
                            TextboxInputState::recv_keyboard_or_gamepad_event(
                                ui_manager,
                                text_measurer,
                                &hover_ui_id,
                                &hover_node_id,
                                UiInputEvent::EndPressed(Modifiers::default()),
                            );
                        }
                        _ => {}
                    }
                }
            }
            UiInputEvent::SelectReleased => {
                if let Some((active_asset_id, active_node_id)) = ui_manager.ui_input_state().get_active_node() {
                    match ui_manager.ui_config(&active_asset_id).unwrap().get_node(&active_node_id).unwrap().widget_kind() {
                        WidgetKind::Button => {
                            ui_manager.ui_input_state_mut().set_active_node(None);
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
    ui_manager: &mut dyn UiManagerTrait,
    ui_asset_id: &AssetId,
    node_id: &NodeId,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some(visible) = ui_manager.ui_state(ui_asset_id).visibility_store.get_node_visibility(node_id) else {
        warn!("no node for id: {:?}", node_id);
        return;
    };
    if !visible {
        return;
    }
    let Some((width, height, child_offset_x, child_offset_y, _)) = ui_manager.ui_state(ui_asset_id).cache.bounds(node_id) else {
        warn!("no bounds for id 2: {:?}", node_id);
        return;
    };

    let Some(node) = ui_manager.ui_config(ui_asset_id).unwrap().get_node(&node_id) else {
        warn!("no node for id: {:?}", node_id);
        return;
    };
    match node.widget_kind() {
        WidgetKind::Button => {
            if point_is_inside(
                (width, height, child_offset_x, child_offset_y),
                mouse_x,
                mouse_y,
            ) {
                ui_manager.ui_input_state_mut().set_cursor_icon(CursorIcon::Hand);
                ui_manager.ui_input_state_mut().receive_hover(ui_asset_id, node_id);
            }
        },
        WidgetKind::Textbox => {

            let is_hovering_eye = ui_manager
                .textbox_receive_hover(
                    ui_asset_id,
                    node_id,
                    (width, height, child_offset_x, child_offset_y),
                    mouse_x,
                    mouse_y
                );

            if point_is_inside(
                (width, height, child_offset_x, child_offset_y),
                mouse_x,
                mouse_y,
            ) {
                ui_manager.ui_input_state_mut().receive_hover(ui_asset_id, node_id);
                if is_hovering_eye {
                    ui_manager.ui_input_state_mut().set_cursor_icon(CursorIcon::Hand);
                } else {
                    ui_manager.ui_input_state_mut().set_cursor_icon(CursorIcon::Text);
                }
            }
        },
        _ => {}
    }
}
