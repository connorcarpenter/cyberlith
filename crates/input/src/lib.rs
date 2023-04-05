pub mod common_conditions;
mod input;
pub mod keyboard;
pub mod mouse;

pub use input::*;

pub mod prelude {
    pub use crate::{
        keyboard::{KeyCode, ScanCode},
        mouse::MouseButton,
        Input,
    };
}

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use keyboard::{keyboard_input_system, KeyCode, KeyboardInput, ScanCode};
use mouse::{
    mouse_button_input_system, MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit,
    MouseWheel,
};

/// Adds keyboard and mouse input to an App
#[derive(Default)]
pub struct InputPlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct InputSystem;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(InputSystem.in_base_set(CoreSet::PreUpdate))
            // keyboard
            .add_event::<KeyboardInput>()
            .init_resource::<Input<KeyCode>>()
            .init_resource::<Input<ScanCode>>()
            .add_system(keyboard_input_system.in_set(InputSystem))
            // mouse
            .add_event::<MouseButtonInput>()
            .add_event::<MouseMotion>()
            .add_event::<MouseWheel>()
            .init_resource::<Input<MouseButton>>()
            .add_system(mouse_button_input_system.in_set(InputSystem));
    }
}

/// The current "press" state of an element
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl ButtonState {
    pub fn is_pressed(&self) -> bool {
        matches!(self, ButtonState::Pressed)
    }
}
