
use crate::gamepad::gamepad::{GamepadAxisType, GamepadButtonType};

pub fn convert_button(button: gilrs::Button) -> Option<GamepadButtonType> {
    match button {
        gilrs::Button::South => Some(GamepadButtonType::South),
        gilrs::Button::East => Some(GamepadButtonType::East),
        gilrs::Button::North => Some(GamepadButtonType::North),
        gilrs::Button::West => Some(GamepadButtonType::West),
        gilrs::Button::C => Some(GamepadButtonType::C),
        gilrs::Button::Z => Some(GamepadButtonType::Z),
        gilrs::Button::LeftTrigger => Some(GamepadButtonType::LeftBumper),
        gilrs::Button::RightTrigger => Some(GamepadButtonType::RightBumper),
        gilrs::Button::Select => Some(GamepadButtonType::Mode),
        gilrs::Button::Start => Some(GamepadButtonType::LeftThumb),
        gilrs::Button::Mode => Some(GamepadButtonType::Mode),
        gilrs::Button::LeftThumb => Some(GamepadButtonType::RightThumb),
        gilrs::Button::RightThumb => Some(GamepadButtonType::RightThumb),
        gilrs::Button::DPadUp => Some(GamepadButtonType::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButtonType::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButtonType::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButtonType::DPadRight),
        gilrs::Button::LeftTrigger2 => Some(GamepadButtonType::Select),
        gilrs::Button::RightTrigger2 => Some(GamepadButtonType::Start),
        gilrs::Button::Unknown => None,
    }
}

pub fn convert_axis(axis: gilrs::Axis, raw_code: u32) -> Option<GamepadAxisType> {
    match axis {
        gilrs::Axis::LeftStickX => Some(GamepadAxisType::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxisType::LeftStickY),
        gilrs::Axis::RightStickX => Some(GamepadAxisType::LeftTrigger),
        gilrs::Axis::RightStickY => Some(GamepadAxisType::RightStickX),
        gilrs::Axis::Unknown => {
            match raw_code {
                46 => Some(GamepadAxisType::RightStickY),
                47 => Some(GamepadAxisType::RightTrigger),
                _ => None,
            }
        },
        gilrs::Axis::LeftZ | gilrs::Axis::RightZ => None,
        // The `axis_dpad_to_button` gilrs filter should filter out all DPadX and DPadY events. If
        // it doesn't then we probably need an entry added to the following repo and an update to
        // GilRs to use the updated database: https://github.com/gabomdq/SDL_GameControllerDB
        gilrs::Axis::DPadX | gilrs::Axis::DPadY => None,
    }
}

pub fn axis_dpad_to_button_filter(ev: Option<gilrs::Event>, gilrs: &mut gilrs::Gilrs) -> Option<gilrs::Event> {

    let ev = ev?;
    let gamepad = gilrs.gamepad(ev.id);

    let btn_dpad_left_code = gamepad.button_code(gilrs::Button::DPadLeft).unwrap();
    let btn_dpad_right_code = gamepad.button_code(gilrs::Button::DPadRight).unwrap();
    let btn_dpad_up_code = gamepad.button_code(gilrs::Button::DPadUp).unwrap();
    let btn_dpad_down_code = gamepad.button_code(gilrs::Button::DPadDown).unwrap();

    match ev.event {
        gilrs::EventType::AxisChanged(gilrs::Axis::Unknown, val, code) => {
            match code.into_u32() {
                48 => { // DPadXAxis
                    let mut release_left = false;
                    let mut release_right = false;
                    let mut event = None;

                    if val == 1.0 {
                        // The axis value might change from left (-1.0) to right (1.0) immediately without
                        // us getting an additional event for the release at the center position (0.0).
                        release_left = gamepad.state().is_pressed(btn_dpad_left_code);

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadRight,
                                1.0,
                                btn_dpad_right_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonPressed(gilrs::Button::DPadRight, btn_dpad_right_code),
                            ..ev
                        });
                    } else if val == -1.0 {
                        // The axis value might change from right (1.0) to left (-1.0) immediately without
                        // us getting an additional event for the release at the center position (0.0).
                        release_right = gamepad.state().is_pressed(btn_dpad_right_code);

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadLeft,
                                1.0,
                                btn_dpad_left_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonPressed(gilrs::Button::DPadLeft, btn_dpad_left_code),
                            ..ev
                        });
                    } else {
                        release_left = gamepad.state().is_pressed(btn_dpad_left_code);
                        release_right = gamepad.state().is_pressed(btn_dpad_right_code);
                    }

                    if release_right {
                        if let Some(event) = event.take() {
                            gilrs.insert_event(event);
                        }

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadRight,
                                0.0,
                                btn_dpad_right_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonReleased(gilrs::Button::DPadRight, btn_dpad_right_code),
                            ..ev
                        });
                    }

                    if release_left {
                        if let Some(event) = event.take() {
                            gilrs.insert_event(event);
                        }

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadLeft,
                                0.0,
                                btn_dpad_left_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonReleased(gilrs::Button::DPadLeft, btn_dpad_left_code),
                            ..ev
                        });
                    }

                    event
                }
                49 => { // DPadYAxis
                    let mut release_up = false;
                    let mut release_down = false;
                    let mut event = None;

                    if val == -1.0 {
                        // The axis value might change from down (-1.0) to up (1.0) immediately without us
                        // getting an additional event for the release at the center position (0.0).
                        release_down = gamepad.state().is_pressed(btn_dpad_down_code);

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadUp, 1.0, btn_dpad_up_code),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonPressed(gilrs::Button::DPadUp, btn_dpad_up_code),
                            ..ev
                        });
                    } else if val == 1.0 {
                        // The axis value might change from up (1.0) to down (-1.0) immediately without us
                        // getting an additional event for the release at the center position (0.0).
                        release_up = gamepad.state().is_pressed(btn_dpad_up_code);

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadDown,
                                1.0,
                                btn_dpad_down_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonPressed(gilrs::Button::DPadDown, btn_dpad_down_code),
                            ..ev
                        });
                    } else {
                        release_up = gamepad.state().is_pressed(btn_dpad_up_code);
                        release_down = gamepad.state().is_pressed(btn_dpad_down_code);
                    }

                    if release_up {
                        if let Some(event) = event.take() {
                            gilrs.insert_event(event);
                        }

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(gilrs::Button::DPadUp, 0.0, btn_dpad_up_code),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonReleased(gilrs::Button::DPadUp, btn_dpad_up_code),
                            ..ev
                        });
                    }

                    if release_down {
                        if let Some(event) = event.take() {
                            gilrs.insert_event(event);
                        }

                        gilrs.insert_event(gilrs::Event {
                            event: gilrs::EventType::ButtonChanged(
                                gilrs::Button::DPadDown,
                                0.0,
                                btn_dpad_down_code,
                            ),
                            ..ev
                        });
                        event = Some(gilrs::Event {
                            event: gilrs::EventType::ButtonReleased(gilrs::Button::DPadDown, btn_dpad_down_code),
                            ..ev
                        });
                    }

                    event
                }
                _ => Some(ev),
            }

        }
        _ => Some(ev),
    }
}