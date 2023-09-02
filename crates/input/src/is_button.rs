use std::collections::{HashMap, HashSet};
use math::Vec2;

use crate::{Key, MouseButton};

pub trait IsButton {
    fn is_pressed(&self, mouse_buttons: &HashMap<MouseButton, Vec2>, key_buttons: &HashSet<Key>) -> bool;
}

impl IsButton for MouseButton {
    fn is_pressed(&self, mouse_buttons: &HashMap<MouseButton, Vec2>, _: &HashSet<Key>) -> bool {
        mouse_buttons.contains_key(self)
    }
}

impl IsButton for Key {
    fn is_pressed(&self, _: &HashMap<MouseButton, Vec2>, key_buttons: &HashSet<Key>) -> bool {
        key_buttons.contains(self)
    }
}
