use crate::GamepadId;

/// An array of every [`GamepadButtonType`] variant.
pub(crate) const ALL_BUTTON_TYPES: [GamepadButtonType; 17] = [
    GamepadButtonType::South,
    GamepadButtonType::East,
    GamepadButtonType::North,
    GamepadButtonType::West,
    GamepadButtonType::LeftTrigger,
    GamepadButtonType::RightTrigger,
    GamepadButtonType::LeftBumper,
    GamepadButtonType::RightBumper,
    GamepadButtonType::Select,
    GamepadButtonType::Start,
    GamepadButtonType::Mode,
    GamepadButtonType::LeftThumb,
    GamepadButtonType::RightThumb,
    GamepadButtonType::DPadUp,
    GamepadButtonType::DPadDown,
    GamepadButtonType::DPadLeft,
    GamepadButtonType::DPadRight,
];

/// A type of a [`GamepadButton`].
///
/// ## Usage
///
/// This is used to determine which button has changed its value when receiving a
/// [`GamepadButtonChangedEvent`]. It is also used in the [`GamepadButton`]
/// which in turn is used to create the [`ButtonInput<GamepadButton>`] or
/// [`Axis<GamepadButton>`] `bevy` resources.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GamepadButtonType {
    /// The bottom action button of the action pad (i.e. PS: Cross, Xbox: A).
    South,
    /// The right action button of the action pad (i.e. PS: Circle, Xbox: B).
    East,
    /// The upper action button of the action pad (i.e. PS: Triangle, Xbox: Y).
    North,
    /// The left action button of the action pad (i.e. PS: Square, Xbox: X).
    West,

    /// The left trigger.
    LeftTrigger,
    /// The right trigger.
    RightTrigger,

    /// The left bumper.
    LeftBumper,
    /// The right bumper.
    RightBumper,

    /// The select button.
    Select,
    /// The start button.
    Start,
    /// The mode button.
    Mode,

    /// The left thumb stick button.
    LeftThumb,
    /// The right thumb stick button.
    RightThumb,

    /// The up button of the D-Pad.
    DPadUp,
    /// The down button of the D-Pad.
    DPadDown,
    /// The left button of the D-Pad.
    DPadLeft,
    /// The right button of the D-Pad.
    DPadRight,

    /// Miscellaneous buttons, considered non-standard (i.e. Extra buttons on a flight stick that do not have a gamepad equivalent).
    Other(u8),
}

/// A button of a [`GamepadId`].
///
/// ## Usage
///
/// It is used as the generic `T` value of an [`ButtonInput`] and [`Axis`] to create `bevy` resources. These
/// resources store the data of the buttons of a gamepad and can be accessed inside of a system.
///
/// ## Updating
///
/// The gamepad button resources are updated inside of the [`gamepad_button_event_system`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GamepadButton {
    /// The gamepad on which the button is located on.
    pub gamepad: GamepadId,
    /// The type of the button.
    pub button_type: GamepadButtonType,
}

impl GamepadButton {
    pub fn new(gamepad: GamepadId, button_type: GamepadButtonType) -> Self {
        Self {
            gamepad,
            button_type,
        }
    }
}
