
mod gamepad;
pub(crate) use gamepad::Gamepads;
pub use gamepad::{GamepadInfo, GamepadId};

mod rumble;
pub use rumble::{RumbleManager, GamepadRumbleIntensity};

mod plugin;
pub(crate) use plugin::GamepadPlugin;

mod settings;
pub use settings::GamepadSettings;

mod axis;
pub(crate) use axis::{Axis, ALL_AXIS_TYPES, GamepadAxisType, GamepadAxis, };
pub use axis::{Joystick, JoystickType};

mod button;
pub(crate) use button::ALL_BUTTON_TYPES;
pub use button::{GamepadButton, GamepadButtonType};

mod converter;
mod error;
mod gilrs;

