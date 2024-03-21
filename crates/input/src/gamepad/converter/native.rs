

pub use gilrs::ev::filter::axis_dpad_to_button as axis_dpad_to_button_filter;

use crate::gamepad::{GamepadAxisType, GamepadButtonType};

pub fn convert_button(button: gilrs::Button) -> Option<GamepadButtonType> {
    match button {
        // face buttons
        gilrs::Button::South => Some(GamepadButtonType::South),
        gilrs::Button::East => Some(GamepadButtonType::East),
        gilrs::Button::North => Some(GamepadButtonType::West),
        gilrs::Button::West => Some(GamepadButtonType::North),

        // triggers
        gilrs::Button::LeftTrigger2 => Some(GamepadButtonType::LeftTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButtonType::RightTrigger),

        // bumpers
        gilrs::Button::LeftTrigger => Some(GamepadButtonType::LeftBumper),
        gilrs::Button::RightTrigger => Some(GamepadButtonType::RightBumper),

        // center buttons
        gilrs::Button::Select => Some(GamepadButtonType::Select),
        gilrs::Button::Start => Some(GamepadButtonType::Start),
        gilrs::Button::Mode => Some(GamepadButtonType::Mode),

        // thumbstick buttons
        gilrs::Button::LeftThumb => Some(GamepadButtonType::LeftThumb),
        gilrs::Button::RightThumb => Some(GamepadButtonType::RightThumb),

        // dpad
        gilrs::Button::DPadUp => Some(GamepadButtonType::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButtonType::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButtonType::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButtonType::DPadRight),

        // ignore
        gilrs::Button::Unknown | gilrs::Button::C | gilrs::Button::Z  => None,
    }
}

pub fn convert_axis(axis: gilrs::Axis, _raw_code: u32) -> Option<GamepadAxisType> {
    match axis {
        // left stick
        gilrs::Axis::LeftStickX => Some(GamepadAxisType::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxisType::LeftStickY),

        // right stick
        gilrs::Axis::RightStickX => Some(GamepadAxisType::RightStickX),
        gilrs::Axis::RightStickY => Some(GamepadAxisType::RightStickY),

        // ignore
        gilrs::Axis::Unknown | gilrs::Axis::DPadX | gilrs::Axis::DPadY | gilrs::Axis::LeftZ | gilrs::Axis::RightZ => None,
    }
}

// Triggers to Button
const PRESS_THRESHOLD: f32 = -0.6;
const RELEASE_THRESHOLD: f32 = -0.9;

pub fn axis_triggers_to_button_filter(ev: Option<gilrs::Event>, gilrs: &mut gilrs::Gilrs) -> Option<gilrs::Event> {

    let ev = ev?;
    let gamepad = gilrs.gamepad(ev.id);

    match ev.event {
        gilrs::EventType::AxisChanged(gilrs::Axis::LeftZ, val, code) => {

            let left_trigger_button = gilrs::Button::LeftTrigger2;

            let is_pressed = gamepad.state().is_pressed(code);
            if is_pressed {
                if val < RELEASE_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            left_trigger_button,
                            0.0,
                            code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonReleased(left_trigger_button, code),
                        ..ev
                    });
                } else {
                    return None;
                }
            } else {
                if val > PRESS_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            left_trigger_button,
                            1.0,
                            code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonPressed(left_trigger_button, code),
                        ..ev
                    });
                } else {
                    return None;
                }
            }
        }
        gilrs::EventType::AxisChanged(gilrs::Axis::RightZ, val, code) => {

            let right_trigger_button = gilrs::Button::RightTrigger2;

            let is_pressed = gamepad.state().is_pressed(code);
            if is_pressed {
                if val < RELEASE_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            right_trigger_button,
                            0.0,
                            code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonReleased(right_trigger_button, code),
                        ..ev
                    });
                } else {
                    return None;
                }
            } else {
                if val > PRESS_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            right_trigger_button,
                            1.0,
                            code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonPressed(right_trigger_button, code),
                        ..ev
                    });
                } else {
                    return None;
                }
            }
        }
        _ => Some(ev),
    }
}