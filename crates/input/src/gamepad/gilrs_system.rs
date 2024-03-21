use bevy_ecs::{system::{NonSend, NonSendMut, Res, ResMut}, event::EventWriter};

use gilrs::{ev::filter::axis_dpad_to_button, EventType, Filter, Gilrs};

use crate::gamepad::{axis::Axis, converter::{convert_axis, convert_button, convert_gamepad_id}, gamepad::{
    GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadConnection, GamepadConnectionEvent,
    GamepadSettings, GamepadEvent, GamepadInfo, GamepadAxis, GamepadButton
}, InputGilrs};

pub fn gilrs_event_startup_system(
    input_gilrs: NonSend<InputGilrs>,
    mut connection_events: EventWriter<GamepadConnectionEvent>,
) {
    for (id, gamepad) in input_gilrs.gilrs.gamepads() {
        let info = GamepadInfo {
            name: gamepad.name().into(),
        };

        connection_events.send(GamepadConnectionEvent {
            gamepad: convert_gamepad_id(id),
            connection: GamepadConnection::Connected(info),
        });
    }
}

pub fn gilrs_event_system(
    mut input_gilrs: NonSendMut<InputGilrs>,
    mut events: EventWriter<GamepadEvent>,
    mut gamepad_buttons: ResMut<Axis<GamepadButton>>,
    gamepad_axis: Res<Axis<GamepadAxis>>,
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

                events.send(
                    GamepadConnectionEvent::new(gamepad, GamepadConnection::Connected(info)).into(),
                );
            }
            EventType::Disconnected => {
                events.send(
                    GamepadConnectionEvent::new(gamepad, GamepadConnection::Disconnected).into(),
                );
            }
            EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                if let Some(button_type) = convert_button(gilrs_button) {
                    let button = GamepadButton::new(gamepad, button_type);
                    let old_value = gamepad_buttons.get(button);
                    let button_settings = gamepad_settings.get_button_axis_settings(button);

                    // Only send events that pass the user-defined change threshold
                    if let Some(filtered_value) = button_settings.filter(raw_value, old_value) {
                        events.send(
                            GamepadButtonChangedEvent::new(gamepad, button_type, filtered_value)
                                .into(),
                        );
                        // Update the current value prematurely so that `old_value` is correct in
                        // future iterations of the loop.
                        gamepad_buttons.set(button, filtered_value);
                    }
                }
            }
            EventType::AxisChanged(gilrs_axis, raw_value, _) => {
                if let Some(axis_type) = convert_axis(gilrs_axis) {
                    let axis = GamepadAxis::new(gamepad, axis_type);
                    let old_value = gamepad_axis.get(axis);
                    let axis_settings = gamepad_settings.get_axis_settings(axis);

                    // Only send events that pass the user-defined change threshold
                    if let Some(filtered_value) = axis_settings.filter(raw_value, old_value) {
                        events.send(
                            GamepadAxisChangedEvent::new(gamepad, axis_type, filtered_value).into(),
                        );
                    }
                }
            }
            _ => (),
        };
    }
    gilrs.inc();
}
