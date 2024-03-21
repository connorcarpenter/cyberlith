use bevy_ecs::system::{NonSend, NonSendMut, Res, ResMut};

use gilrs::{ev::filter::axis_dpad_to_button, EventType, Filter};

use crate::{gamepad::{converter::{convert_axis, convert_button, convert_gamepad_id}, gamepad::{
    GamepadSettings, GamepadInfo, GamepadAxis, GamepadButton
}, InputGilrs}, Input};

pub fn gilrs_event_startup_system(
    input_gilrs: NonSend<InputGilrs>,
    mut input_winit: ResMut<Input>,
) {
    for (id, gamepad) in input_gilrs.gilrs.gamepads() {
        let info = GamepadInfo {
            name: gamepad.name().into(),
        };

        input_winit.recv_gilrs_gamepad_connect(convert_gamepad_id(id), info);
    }
}

pub fn gilrs_event_system(
    mut input_gilrs: NonSendMut<InputGilrs>,
    mut input_winit: ResMut<Input>,
    gamepad_settings: Res<GamepadSettings>,
) {
    let mut gilrs = &mut input_gilrs.gilrs;
    while let Some(gilrs_event) = gilrs
        .next_event()
        .filter_ev(&axis_dpad_to_button, &mut gilrs)
    {
        gilrs.update(&gilrs_event);

        let gamepad = convert_gamepad_id(gilrs_event.id);
        match gilrs_event.event {
            EventType::Connected => {
                let pad = gilrs.gamepad(gilrs_event.id);
                let info = GamepadInfo {
                    name: pad.name().into(),
                };

                input_winit.recv_gilrs_gamepad_connect(gamepad, info);
            }
            EventType::Disconnected => {
                input_winit.recv_gilrs_gamepad_disconnect(gamepad);
            }
            EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                if let Some(button_type) = convert_button(gilrs_button) {
                    let button = GamepadButton::new(gamepad, button_type);
                    let old_value = input_winit.gamepad_button_axis_get(button);
                    let button_settings = gamepad_settings.get_button_axis_settings(button);

                    // Only send events that pass the user-defined change threshold
                    if let Some(filtered_value) = button_settings.filter(raw_value, old_value) {

                        {
                            let button = GamepadButton::new(gamepad, button_type);
                            let value = filtered_value;
                            let button_property = gamepad_settings.get_button_settings(button);

                            if button_property.is_released(value) {
                                // Check if button was previously pressed
                                if input_winit.gamepad_button_is_pressed(button) {
                                    input_winit.recv_gilrs_button_release(gamepad, button_type);
                                }
                                // We don't have to check if the button was previously pressed here
                                // because that check is performed within Input<T>::release()
                                input_winit.gamepad_button_release(button);
                            } else if button_property.is_pressed(value) {
                                // Check if button was previously not pressed
                                if !input_winit.gamepad_button_is_pressed(button) {
                                    input_winit.recv_gilrs_button_press(gamepad, button_type);
                                }
                                input_winit.gamepad_button_press(button);
                            };
                        }

                        // Update the current value prematurely so that `old_value` is correct in
                        // future iterations of the loop.
                        input_winit.gamepad_button_axis_set(button, filtered_value);
                    }
                }
            }
            EventType::AxisChanged(gilrs_axis, raw_value, _) => {
                if let Some(axis_type) = convert_axis(gilrs_axis) {
                    let axis = GamepadAxis::new(gamepad, axis_type);
                    let old_value = input_winit.gamepad_axis_get(axis);
                    let axis_settings = gamepad_settings.get_axis_settings(axis);

                    // Only send events that pass the user-defined change threshold
                    if let Some(filtered_value) = axis_settings.filter(raw_value, old_value) {
                        let axis = GamepadAxis::new(gamepad, axis_type);
                        input_winit.gamepad_axis_set(axis, filtered_value);
                    }
                }
            }
            _ => (),
        };
    }
    gilrs.inc();
}
