use std::time::Duration;
use bevy_ecs::event::EventWriter;

use bevy_ecs::system::{NonSend, NonSendMut, ResMut};

use gilrs::{ff, EventType, Filter, Gilrs};

use crate::{gamepad::{
    converter::{
        axis_dpad_to_button_filter, axis_triggers_to_button_filter, convert_axis,
        convert_button, convert_gamepad_id,
    },
    rumble,
    rumble::RunningRumbleEffects,
    GamepadAxis, GamepadButton, GamepadInfo,
}, GamepadId, GamepadRumbleIntensity, Input, InputEvent};

pub struct GilrsWrapper {
    gilrs: Gilrs,

    // this is in here and not the RumbleManager because it is non-Send
    running_rumbles: RunningRumbleEffects,
}

impl GilrsWrapper {
    pub fn new(gilrs: Gilrs) -> Self {
        Self {
            gilrs,
            running_rumbles: RunningRumbleEffects::default(),
        }
    }

    pub fn gilrs_mut(&mut self) -> &mut Gilrs {
        &mut self.gilrs
    }

    pub fn update_rumbles(&mut self) {
        if let Some(updated_gamepads) = self.running_rumbles.update() {
            let _result = rumble::set_total_rumbles(self, updated_gamepads);
            // TODO: handle result
        }
        self.running_rumbles.cleanup();
    }

    // unused on native, used on wasm
    #[allow(unused)]
    pub fn get_current_rumble(
        &self,
        gamepad_id: &GamepadId,
    ) -> Option<(Duration, GamepadRumbleIntensity)> {
        let output = self.running_rumbles.get_current_rumble(gamepad_id);

        output
    }

    pub fn add_rumble(
        &mut self,
        gamepad: &GamepadId,
        duration: Duration,
        intensity: GamepadRumbleIntensity,
        effect: Option<ff::Effect>,
    ) {
        self.running_rumbles
            .add_rumble(gamepad, duration, intensity, effect);
    }

    // used as a system
    pub fn startup(
        input_gilrs: NonSend<GilrsWrapper>,
        mut input: ResMut<Input>,
        mut event_writer: EventWriter<InputEvent>
    ) {
        for (id, gamepad) in input_gilrs.gilrs.gamepads() {
            let info = GamepadInfo {
                name: gamepad.name().into(),
            };

            input.recv_gilrs_gamepad_connect(&mut event_writer, convert_gamepad_id(id), info);
        }
    }

    // used as a system
    pub fn update(mut gilrs_wrapper: NonSendMut<GilrsWrapper>, mut input: ResMut<Input>, mut event_writer: EventWriter<InputEvent>) {
        let mut gilrs = &mut gilrs_wrapper.gilrs;

        while let Some(gilrs_event) = gilrs.next_event() {
            // info!("---");
            // info!("GILRS raw event: {:?}", gilrs_event);

            let Some(gilrs_event) = gilrs_event
                .filter_ev(&axis_dpad_to_button_filter, &mut gilrs)
                .filter_ev(&axis_triggers_to_button_filter, &mut gilrs)
            else {
                // info!("GILRS fil event: NONE");
                continue;
            };

            // info!("GILRS fil event: {:?}", gilrs_event);

            gilrs.update(&gilrs_event);

            let gamepad = convert_gamepad_id(gilrs_event.id);

            match gilrs_event.event {
                EventType::Connected => {
                    let pad = gilrs.gamepad(gilrs_event.id);
                    let info = GamepadInfo {
                        name: pad.name().into(),
                    };

                    input.recv_gilrs_gamepad_connect(&mut event_writer, gamepad, info);
                }
                EventType::Disconnected => {
                    input.recv_gilrs_gamepad_disconnect(&mut event_writer, gamepad);
                }
                EventType::ButtonChanged(gilrs_button, raw_value, _) => {
                    if let Some(button_type) = convert_button(gilrs_button) {
                        let button = GamepadButton::new(gamepad, button_type);
                        let old_value = input.gamepad_button_axis_get(button);
                        let button_settings =
                            input.gamepad_settings().get_button_axis_settings(button);

                        // Only send events that pass the user-defined change threshold
                        if let Some(filtered_value) = button_settings.filter(raw_value, old_value) {
                            {
                                let button = GamepadButton::new(gamepad, button_type);
                                let value = filtered_value;
                                let button_property =
                                    input.gamepad_settings().get_button_settings(button);

                                if button_property.is_released(value) {
                                    // Check if button was previously pressed
                                    if input.is_pressed(button) {
                                        input.recv_gilrs_button_release(&mut event_writer, gamepad, button_type);
                                    }
                                    // We don't have to check if the button was previously pressed here
                                    // because that check is performed within Input<T>::release()
                                    input.gamepad_button_release(button);
                                } else if button_property.is_pressed(value) {
                                    // Check if button was previously not pressed
                                    if !input.is_pressed(button) {
                                        input.recv_gilrs_button_press(&mut event_writer, gamepad, button_type);
                                    }
                                    input.gamepad_button_press(button);
                                };
                            }

                            // Update the current value prematurely so that `old_value` is correct in
                            // future iterations of the loop.
                            input.gamepad_button_axis_set(button, filtered_value);
                        }
                    }
                }
                EventType::AxisChanged(gilrs_axis, raw_value, code) => {
                    if let Some(axis_type) = convert_axis(gilrs_axis, code.into_u32()) {
                        let axis = GamepadAxis::new(gamepad, axis_type);
                        let old_value = input.gamepad_axis_get(axis);
                        let axis_settings = input.gamepad_settings().get_axis_settings(axis);

                        // Only send events that pass the user-defined change threshold
                        if let Some(filtered_value) = axis_settings.filter(raw_value, old_value) {
                            let axis = GamepadAxis::new(gamepad, axis_type);
                            input.gamepad_axis_set(&mut event_writer, axis, filtered_value);
                        }
                    }
                }
                _ => (),
            };
        }
        gilrs.inc();
    }
}
