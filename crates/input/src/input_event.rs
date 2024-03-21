use bevy_ecs::event::Event;

use math::Vec2;

use crate::{Key, MouseButton, gamepad::{GamepadId, GamepadAxisType, GamepadButtonType}};

#[derive(Clone, Copy, Event)]
pub enum InputEvent {
    // mouse
    MouseClicked(MouseButton, Vec2),
    MouseReleased(MouseButton),
    MouseMoved(Vec2),
    MouseDragged(MouseButton, Vec2, Vec2),
    MouseMiddleScrolled(f32),
    // keyboard
    KeyPressed(Key),
    KeyReleased(Key),
    // gamepad
    GamepadConnected(GamepadId),
    GamepadDisconnected(GamepadId),
    GamepadButtonPressed(GamepadId, GamepadButtonType),
    GamepadButtonReleased(GamepadId, GamepadButtonType),
    GamepadAxisChanged(GamepadId, GamepadAxisType, f32),
}
