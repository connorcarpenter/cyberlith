
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
pub(crate) use axis::{Axis, ALL_AXIS_TYPES};
pub use axis::{GamepadAxisType, GamepadAxis};

mod button;
pub(crate) use button::ALL_BUTTON_TYPES;
pub use button::{GamepadButton, GamepadButtonType};

mod converter;
mod error;
mod gilrs;

