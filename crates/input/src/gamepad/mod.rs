mod gamepad;
pub(crate) use gamepad::Gamepads;
pub use gamepad::{GamepadId, GamepadInfo};

mod rumble;
pub use rumble::{GamepadRumbleIntensity, RumbleManager};

mod plugin;
pub(crate) use plugin::GamepadPlugin;

mod settings;
pub use settings::GamepadSettings;

mod axis;
pub(crate) use axis::{Axis, GamepadAxis, GamepadAxisType, ALL_AXIS_TYPES};
pub use axis::{Joystick, JoystickType};

mod button;
pub(crate) use button::ALL_BUTTON_TYPES;
pub use button::{GamepadButton, GamepadButtonType};

mod converter;
mod error;
mod gilrs;
