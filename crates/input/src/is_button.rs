use std::collections::HashSet;

use crate::{Key, MouseButton};

pub trait IsButton {
    fn is_pressed(&self, mouse_buttons: &HashSet<MouseButton>, key_buttons: &HashSet<Key>) -> bool;
}

impl IsButton for MouseButton {
    fn is_pressed(&self, mouse_buttons: &HashSet<MouseButton>, _: &HashSet<Key>) -> bool {
        mouse_buttons.contains(self)
    }
}

impl IsButton for Key {
    fn is_pressed(&self, _: &HashSet<MouseButton>, key_buttons: &HashSet<Key>) -> bool {
        key_buttons.contains(self)
    }
}
