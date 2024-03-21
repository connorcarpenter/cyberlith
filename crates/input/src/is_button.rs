use std::collections::HashSet;

use crate::{GamepadButton, Key, MouseButton};

pub trait IsButton {
    fn is_pressed(
        &self,
        mouse_buttons: &HashSet<MouseButton>,
        key_buttons: &HashSet<Key>,
        gamepad_buttons: &HashSet<GamepadButton>,
    ) -> bool;
}

impl IsButton for MouseButton {
    fn is_pressed(
        &self,
        mouse_buttons: &HashSet<MouseButton>,
        _: &HashSet<Key>,
        _: &HashSet<GamepadButton>,
    ) -> bool {
        mouse_buttons.contains(self)
    }
}

impl IsButton for Key {
    fn is_pressed(
        &self,
        _: &HashSet<MouseButton>,
        key_buttons: &HashSet<Key>,
        _: &HashSet<GamepadButton>,
    ) -> bool {
        key_buttons.contains(self)
    }
}

impl IsButton for GamepadButton {
    fn is_pressed(
        &self,
        _: &HashSet<MouseButton>,
        _: &HashSet<Key>,
        gamepad_buttons: &HashSet<GamepadButton>,
    ) -> bool {
        gamepad_buttons.contains(self)
    }
}
