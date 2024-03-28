use bevy_ecs::event::Event;

use math::Vec2;

use crate::{gamepad::{GamepadButtonType, GamepadId}, JoystickType, Key, Modifiers, MouseButton};

#[derive(Clone, Event)]
pub enum InputEvent {
    // mouse
    MouseClicked(MouseButton, Vec2, Modifiers),
    MouseDoubleClicked(MouseButton, Vec2, Modifiers),
    MouseTripleClicked(MouseButton, Vec2, Modifiers),
    MouseReleased(MouseButton),
    MouseMoved(Vec2),
              //(button,  position, delta, modifiers)
    MouseDragged(MouseButton, Vec2, Vec2, Modifiers),
    MouseMiddleScrolled(f32),
    // keyboard
    KeyPressed(Key, Modifiers),
    KeyReleased(Key),
    Text(char),
    // gamepad
    GamepadConnected(GamepadId),
    GamepadDisconnected(GamepadId),
    GamepadButtonPressed(GamepadId, GamepadButtonType),
    GamepadButtonReleased(GamepadId, GamepadButtonType),
    GamepadJoystickMoved(GamepadId, JoystickType, Vec2),
    // clipboard
    Cut,
    Copy,
    Paste(String),
}
