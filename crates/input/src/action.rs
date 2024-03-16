use bevy_ecs::event::Event;

use math::Vec2;

use crate::{Key, MouseButton};

#[derive(Clone, Copy, Event)]
pub enum InputEvent {
    MouseClick(MouseButton, Vec2),
    MouseRelease(MouseButton),
    MouseMoved,
    MouseDragged(MouseButton, Vec2, Vec2),
    MiddleMouseScroll(f32),
    KeyPress(Key),
    KeyRelease(Key),
}
