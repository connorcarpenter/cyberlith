use crate::{Key, Modifiers, MouseButton};

/// An input event (from mouse, keyboard or similar).
#[derive(Clone, Debug)]
pub enum IncomingEvent {
    /// Fired when a button is pressed or the screen is touched.
    MousePress(
        /// Type of button
        MouseButton,
        /// The screen position in logical pixels, to get it in physical pixels, multiply it with the device pixel ratio.
        /// The first value defines the position on the horizontal axis with zero being at the left border of the window
        /// and the second on the vertical axis with zero being at the top edge of the window.
        (f64, f64),
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired when a button is released or the screen is stopped being touched.
    MouseRelease(
        /// Type of button
        MouseButton,
        /// The screen position in logical pixels, to get it in physical pixels, multiply it with the device pixel ratio.
        /// The first value defines the position on the horizontal axis with zero being at the left border of the window
        /// and the second on the vertical axis with zero being at the top edge of the window.
        (f64, f64),
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired continuously when the mouse or a finger on the screen is moved.
    MouseMotion(
        /// Type of button if a button is pressed.
        Option<MouseButton>,
        /// The relative movement of the mouse/finger since last [IncomingEvent::MouseMotion] event.
        (f64, f64),
        /// The screen position in logical pixels, to get it in physical pixels, multiply it with the device pixel ratio.
        /// The first value defines the position on the horizontal axis with zero being at the left border of the window
        /// and the second on the vertical axis with zero being at the top edge of the window.
        (f64, f64),
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired continuously when the mouse wheel or equivalent is applied.
    MouseWheel(
        /// The relative scrolling since the last [IncomingEvent::MouseWheel] event.
        (f64, f64),
        /// The screen position in logical pixels, to get it in physical pixels, multiply it with the device pixel ratio.
        /// The first value defines the position on the horizontal axis with zero being at the left border of the window
        /// and the second on the vertical axis with zero being at the top edge of the window.
        (f64, f64),
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired when a key is pressed.
    KeyPress(
        /// The type of key.
        Key,
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired when a key is released.
    KeyRelease(
        /// The type of key.
        Key,
        /// The state of modifiers.
        Modifiers,
    ),
    /// Fired when the modifiers change.
    ModifiersChange(
        /// The state of modifiers after the change.
        Modifiers,
    ),
    /// Fires when some text has been written.
    Text(String),
}
