

pub use gilrs::ev::filter::axis_dpad_to_button as axis_dpad_to_button_filter;

use crate::gamepad::{GamepadAxisType, GamepadButtonType};

pub fn convert_button(button: gilrs::Button) -> Option<GamepadButtonType> {
    match button {
        gilrs::Button::South => Some(GamepadButtonType::South),
        gilrs::Button::East => Some(GamepadButtonType::East),
        gilrs::Button::North => Some(GamepadButtonType::West),
        gilrs::Button::West => Some(GamepadButtonType::North),
        gilrs::Button::C => Some(GamepadButtonType::C),
        gilrs::Button::Z => Some(GamepadButtonType::Z),
        gilrs::Button::LeftTrigger => Some(GamepadButtonType::LeftBumper),
        gilrs::Button::RightTrigger => Some(GamepadButtonType::RightBumper),
        gilrs::Button::Select => Some(GamepadButtonType::Select),
        gilrs::Button::Start => Some(GamepadButtonType::Start),
        gilrs::Button::Mode => Some(GamepadButtonType::Mode),
        gilrs::Button::LeftThumb => Some(GamepadButtonType::LeftThumb),
        gilrs::Button::RightThumb => Some(GamepadButtonType::RightThumb),
        gilrs::Button::DPadUp => Some(GamepadButtonType::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButtonType::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButtonType::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButtonType::DPadRight),
        gilrs::Button::LeftTrigger2 => None,
        gilrs::Button::RightTrigger2 => None,
        gilrs::Button::Unknown => None,
    }
}

pub fn convert_axis(axis: gilrs::Axis, _raw_code: u32) -> Option<GamepadAxisType> {
    match axis {
        gilrs::Axis::LeftStickX => Some(GamepadAxisType::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxisType::LeftStickY),
        gilrs::Axis::LeftZ => Some(GamepadAxisType::LeftTrigger),
        gilrs::Axis::RightStickX => Some(GamepadAxisType::RightStickX),
        gilrs::Axis::RightStickY => Some(GamepadAxisType::RightStickY),
        gilrs::Axis::RightZ => Some(GamepadAxisType::RightTrigger),
        // The `axis_dpad_to_button` gilrs filter should filter out all DPadX and DPadY events. If
        // it doesn't then we probably need an entry added to the following repo and an update to
        // GilRs to use the updated database: https://github.com/gabomdq/SDL_GameControllerDB
        gilrs::Axis::Unknown | gilrs::Axis::DPadX | gilrs::Axis::DPadY => None,
    }
}