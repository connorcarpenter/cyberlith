use bevy_ecs::{event::EventReader, system::{Res, ResMut, SystemParam}};

use input::{
    keyboard::{KeyCode, KeyboardInput},
    mouse::{MouseButton, MouseButtonInput, MouseScrollUnit, MouseWheel},
    ButtonState, Input,
};
use window::{
    CursorEntered, CursorLeft, CursorMoved, ReceivedCharacter, RequestRedraw, WindowCreated,
    WindowFocused,
};

use crate::resources::{EguiMousePosition, EguiSettings};

#[derive(SystemParam)]
pub struct InputEvents<'w, 's> {
    pub ev_cursor_entered: EventReader<'w, 's, CursorEntered>,
    pub ev_cursor_left: EventReader<'w, 's, CursorLeft>,
    pub ev_cursor: EventReader<'w, 's, CursorMoved>,
    pub ev_mouse_button_input: EventReader<'w, 's, MouseButtonInput>,
    pub ev_mouse_wheel: EventReader<'w, 's, MouseWheel>,
    pub ev_received_character: EventReader<'w, 's, ReceivedCharacter>,
    pub ev_keyboard_input: EventReader<'w, 's, KeyboardInput>,
    pub ev_window_focused: EventReader<'w, 's, WindowFocused>,
    pub ev_window_created: EventReader<'w, 's, WindowCreated>,
}

/// Processes Bevy input and feeds it to Egui.
pub fn process_input(
    mut input_events: InputEvents,
    input_resources: InputResources,
    mut context_params: ContextSystemParams,
    egui_settings: Res<EguiSettings>,
    mut egui_mouse_position: ResMut<EguiMousePosition>,
    time: Res<Time>,
) {
    // This is a workaround for Windows. For some reason, `WindowFocused` event isn't fired
    // when a window is created.
    if let Some(event) = input_events.ev_window_created.iter().last() {
        *context_params.focused_window = Some(event.window);
    }

    for event in input_events.ev_window_focused.iter() {
        *context_params.focused_window = if event.focused {
            Some(event.window)
        } else {
            None
        };
    }

    let shift = input_resources.keyboard_input.pressed(KeyCode::LShift)
        || input_resources.keyboard_input.pressed(KeyCode::RShift);
    let ctrl = input_resources.keyboard_input.pressed(KeyCode::LControl)
        || input_resources.keyboard_input.pressed(KeyCode::RControl);
    let alt = input_resources.keyboard_input.pressed(KeyCode::LAlt)
        || input_resources.keyboard_input.pressed(KeyCode::RAlt);
    let win = input_resources.keyboard_input.pressed(KeyCode::LWin)
        || input_resources.keyboard_input.pressed(KeyCode::RWin);

    let mac_cmd = if cfg!(target_os = "macos") {
        win
    } else {
        false
    };
    let command = if cfg!(target_os = "macos") { win } else { ctrl };

    let modifiers = egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd,
        command,
    };

    let mut cursor_left_window = None;
    if let Some(cursor_left) = input_events.ev_cursor_left.iter().last() {
        cursor_left_window = Some(cursor_left.window);
    }
    let cursor_entered_window = input_events
        .ev_cursor_entered
        .iter()
        .last()
        .map(|event| event.window);

    // When a user releases a mouse button, Safari emits both `CursorLeft` and `CursorEntered`
    // events during the same frame. We don't want to reset mouse position in such a case, otherwise
    // we won't be able to process the mouse button event.
    let prev_mouse_position =
        if cursor_left_window.is_some() && cursor_left_window != cursor_entered_window {
            // If it's not the Safari edge case, reset the mouse position.
            egui_mouse_position.take()
        } else {
            None
        };

    if let Some(cursor_moved) = input_events.ev_cursor.iter().last() {
        // If we've left the window, it's unlikely that we've moved the cursor back to the same
        // window this exact frame, so we are safe to ignore all `CursorMoved` events for the window
        // that has been left.
        if cursor_left_window != Some(cursor_moved.window) {
            let scale_factor = egui_settings.scale_factor as f32;
            let mut mouse_position: (f32, f32) = (cursor_moved.position / scale_factor).into();
            let mut context = context_params
                .contexts
                .get_mut(cursor_moved.window)
                .unwrap();
            mouse_position.1 = context.window_size.height() / scale_factor - mouse_position.1;
            egui_mouse_position.0 = Some((cursor_moved.window, mouse_position.into()));
            context
                .egui_input
                .events
                .push(egui::Event::PointerMoved(egui::pos2(
                    mouse_position.0,
                    mouse_position.1,
                )));
        }
    }

    // If we pressed a button, started dragging a cursor inside a window and released
    // the button when being outside, some platforms will fire `CursorLeft` again together
    // with `MouseButtonInput` - this is why we also take `prev_mouse_position` into account.
    if let Some((window_id, position)) = egui_mouse_position.or(prev_mouse_position) {
        if let Ok(mut context) = context_params.contexts.get_mut(window_id) {
            let events = &mut context.egui_input.events;

            for mouse_button_event in input_events.ev_mouse_button_input.iter() {
                let button = match mouse_button_event.button {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    _ => None,
                };
                let pressed = match mouse_button_event.state {
                    ButtonState::Pressed => true,
                    ButtonState::Released => false,
                };
                if let Some(button) = button {
                    events.push(egui::Event::PointerButton {
                        pos: position.to_pos2(),
                        button,
                        pressed,
                        modifiers,
                    });
                }
            }

            for event in input_events.ev_mouse_wheel.iter() {
                let mut delta = egui::vec2(event.x, event.y);
                if let MouseScrollUnit::Line = event.unit {
                    // https://github.com/emilk/egui/blob/a689b623a669d54ea85708a8c748eb07e23754b0/egui-winit/src/lib.rs#L449
                    delta *= 50.0;
                }

                if ctrl || mac_cmd {
                    // Treat as zoom instead.
                    let factor = (delta.y / 200.0).exp();
                    events.push(egui::Event::Zoom(factor));
                } else if shift {
                    // Treat as horizontal scrolling.
                    // Note: Mac already fires horizontal scroll events when shift is down.
                    events.push(egui::Event::Scroll(egui::vec2(delta.x + delta.y, 0.0)));
                } else {
                    events.push(egui::Event::Scroll(delta));
                }
            }
        }
    }

    if !command || cfg!(target_os = "windows") && ctrl && alt {
        for event in input_events.ev_received_character.iter() {
            if !event.char.is_control() {
                let mut context = context_params.contexts.get_mut(event.window).unwrap();
                context
                    .egui_input
                    .events
                    .push(egui::Event::Text(event.char.to_string()));
            }
        }
    }

    if let Some(mut focused_input) = context_params
        .focused_window
        .as_ref()
        .and_then(|window_id| {
            if let Ok(context) = context_params.contexts.get_mut(*window_id) {
                Some(context.egui_input)
            } else {
                None
            }
        })
    {
        for ev in input_events.ev_keyboard_input.iter() {
            if let Some(key) = ev.key_code.and_then(bevy_to_egui_key) {
                let pressed = match ev.state {
                    ButtonState::Pressed => true,
                    ButtonState::Released => false,
                };
                let egui_event = egui::Event::Key {
                    key,
                    pressed,
                    repeat: false,
                    modifiers,
                };
                focused_input.events.push(egui_event);

                // We also check that it's an `ButtonState::Pressed` event, as we don't want to
                // copy, cut or paste on the key release.
                #[cfg(feature = "manage_clipboard")]
                if command && pressed {
                    match key {
                        egui::Key::C => {
                            focused_input.events.push(egui::Event::Copy);
                        }
                        egui::Key::X => {
                            focused_input.events.push(egui::Event::Cut);
                        }
                        egui::Key::V => {
                            if let Some(contents) = input_resources.egui_clipboard.get_contents() {
                                focused_input.events.push(egui::Event::Text(contents))
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        focused_input.modifiers = modifiers;
    }

    for mut context in context_params.contexts.iter_mut() {
        context.egui_input.predicted_dt = time.raw_delta_seconds();
    }

    // In some cases, we may skip certain events. For example, we ignore `ReceivedCharacter` events
    // when alt or ctrl button is pressed. We still want to clear event buffer.
    input_events.clear();
}
