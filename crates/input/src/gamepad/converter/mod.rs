
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod wasm;
        pub use self::wasm::*;
    }
    else {
        mod native;
        pub use self::native::*;
    }
}

use crate::GamepadId;

pub fn convert_gamepad_id(gamepad_id: gilrs::GamepadId) -> GamepadId {
    GamepadId::new(gamepad_id.into())
}

const PRESS_THRESHOLD: f32 = -0.6;
const RELEASE_THRESHOLD: f32 = -0.9;

pub fn axis_triggers_to_button_filter(ev: Option<gilrs::Event>, gilrs: &mut gilrs::Gilrs) -> Option<gilrs::Event> {

    let ev = ev?;
    let gamepad = gilrs.gamepad(ev.id);

    match ev.event {
        gilrs::EventType::AxisChanged(gilrs::Axis::LeftZ, val, _) => {
            let btn_left_trigger_code = gamepad.axis_code(gilrs::Axis::LeftZ).unwrap();
            let is_pressed = gamepad.state().is_pressed(btn_left_trigger_code);
            if is_pressed {
                if val < RELEASE_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            gilrs::Button::LeftTrigger2,
                            0.0,
                            btn_left_trigger_code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonReleased(gilrs::Button::LeftTrigger2, btn_left_trigger_code),
                        ..ev
                    });
                } else {
                    return None;
                }
            } else {
                if val > PRESS_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            gilrs::Button::LeftTrigger2,
                            1.0,
                            btn_left_trigger_code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonPressed(gilrs::Button::LeftTrigger2, btn_left_trigger_code),
                        ..ev
                    });
                } else {
                    return None;
                }
            }
        }
        gilrs::EventType::AxisChanged(gilrs::Axis::RightZ, val, _) => {
            let btn_right_trigger_code = gamepad.axis_code(gilrs::Axis::RightZ).unwrap();
            let is_pressed = gamepad.state().is_pressed(btn_right_trigger_code);
            if is_pressed {
                if val < RELEASE_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            gilrs::Button::RightTrigger2,
                            0.0,
                            btn_right_trigger_code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonReleased(gilrs::Button::RightTrigger2, btn_right_trigger_code),
                        ..ev
                    });
                } else {
                    return None;
                }
            } else {
                if val > PRESS_THRESHOLD {
                    gilrs.insert_event(gilrs::Event {
                        event: gilrs::EventType::ButtonChanged(
                            gilrs::Button::RightTrigger2,
                            1.0,
                            btn_right_trigger_code,
                        ),
                        ..ev
                    });
                    return Some(gilrs::Event {
                        event: gilrs::EventType::ButtonPressed(gilrs::Button::RightTrigger2, btn_right_trigger_code),
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