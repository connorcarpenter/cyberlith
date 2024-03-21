//! The gamepad input functionality.

use std::collections::HashMap;

/// A gamepad with an associated `ID`.
///
/// ## Usage
///
/// The primary way to access the individual connected gamepads is done through the [`Gamepads`]
/// `bevy` resource. It is also used inside of [`GamepadConnectionEvent`]s to correspond a gamepad
/// with a connection event.
///
/// ## Note
///
/// The `ID` of a gamepad is fixed until the gamepad disconnects or the app is restarted.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GamepadId {
    /// The `ID` of the gamepad.
    id: usize,
}

impl GamepadId {
    /// Creates a new [`GamepadId`].
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

/// Metadata associated with a [`GamepadId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamepadInfo {
    /// The name of the gamepad.
    ///
    /// This name is generally defined by the OS.
    ///
    /// For example on Windows the name may be "HID-compliant game controller".
    pub name: String,
}

/// A collection of connected [`GamepadId`]s.
///
/// ## Usage
///
/// It is stored in a `bevy` resource which tracks all of the currently connected [`GamepadId`]s.
///
/// ## Updating
///
/// The [`GamepadId`]s are registered and deregistered in the [`gamepad_connection_system`]
/// whenever a [`GamepadConnectionEvent`] is received.
#[derive(Default, Debug)]
pub(crate) struct Gamepads {
    /// The collection of the connected [`GamepadId`]s.
    gamepads: HashMap<GamepadId, GamepadInfo>,
}

impl Gamepads {
    // /// Returns `true` if the `gamepad` is connected.
    // pub fn contains(&self, gamepad: GamepadId) -> bool {
    //     self.gamepads.contains_key(&gamepad)
    // }

    /// Returns an iterator over registered [`GamepadId`]s in an arbitrary order.
    pub fn iter(&self) -> impl Iterator<Item = GamepadId> + '_ {
        self.gamepads.keys().copied()
    }

    // /// The name of the gamepad if this one is connected.
    // pub fn name(&self, gamepad: GamepadId) -> Option<&str> {
    //     self.gamepads.get(&gamepad).map(|g| g.name.as_str())
    // }

    /// Registers the `gamepad`, marking it as connected.
    pub(crate) fn register(&mut self, gamepad: GamepadId, info: GamepadInfo) {
        self.gamepads.insert(gamepad, info);
    }

    /// Deregisters the `gamepad`, marking it as disconnected.
    pub(crate) fn deregister(&mut self, gamepad: GamepadId) {
        self.gamepads.remove(&gamepad);
    }
}