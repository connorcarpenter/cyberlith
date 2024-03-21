
mod gamepad;
pub(crate) use gamepad::{ALL_BUTTON_TYPES, ALL_AXIS_TYPES};
pub use gamepad::{GamepadInfo, Gamepads, GamepadButton, GamepadSettings, GamepadAxis, GamepadId, GamepadButtonType, GamepadAxisType};

mod axis;
pub use axis::{Axis};

mod gilrs_system;

mod rumble;
pub use rumble::{RumbleManager, GamepadRumbleIntensity};

mod converter;
mod error;

mod plugin;
pub use plugin::GilrsPlugin;