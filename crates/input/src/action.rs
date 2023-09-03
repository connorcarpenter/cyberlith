use math::Vec2;

use crate::{Key, MouseButton};

#[derive(Clone, Copy)]
pub enum InputAction {
    MiddleMouseScroll(f32),
    MouseMoved,
    MouseDragged(MouseButton, Vec2, Vec2),
    MouseClick(MouseButton, Vec2),
    MouseRelease(MouseButton),
    KeyPress(Key),
    KeyRelease(Key),
}
