use std::collections::HashSet;

use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, SystemSet};

use gilrs::{Gilrs, GilrsBuilder};

mod converter;

mod gamepad;
pub use gamepad::{GamepadInfo, Gamepads, GamepadButton, GamepadAxis, GamepadId, GamepadButtonType, GamepadAxisType, GamepadAxisChangedEvent, GamepadButtonChangedEvent, GamepadConnectionEvent, GamepadRumbleRequest, GamepadRumbleIntensity, GamepadButtonInput};

mod axis;
pub use axis::{Axis};

mod gilrs_system;
use gilrs_system::{gilrs_event_startup_system, gilrs_event_system};

mod rumble;

use rumble::{play_gilrs_rumble, RunningRumbleEffects};

use crate::gamepad::{gamepad::{gamepad_axis_event_system, gamepad_button_event_system, gamepad_connection_system, gamepad_event_system, GamepadEvent, GamepadSettings}};

/// Plugin that provides gamepad handling to an [`App`].
#[derive(Default)]
pub struct GilrsPlugin;

/// Updates the running gamepad rumble effects.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct RumbleSystem;

impl Plugin for GilrsPlugin {
    fn build(&self, app: &mut App) {

        //
        app.add_event::<GamepadConnectionEvent>()
            .add_event::<GamepadButtonChangedEvent>()
            .add_event::<GamepadButtonInput>()
            .add_event::<GamepadAxisChangedEvent>()
            .add_event::<GamepadEvent>()
            .add_event::<GamepadRumbleRequest>()
            .init_resource::<GamepadSettings>()
            .init_resource::<Gamepads>()
            .init_resource::<Axis<GamepadAxis>>()
            .init_resource::<Axis<GamepadButton>>()
            .add_systems(
                PreUpdate,
                (
                    gamepad_connection_system.after(gamepad_event_system),
                    gamepad_event_system,
                    gamepad_button_event_system
                        .after(gamepad_event_system)
                        .after(gamepad_connection_system),
                    gamepad_axis_event_system
                        .after(gamepad_event_system)
                        .after(gamepad_connection_system),
                ),
            );

        // gilrs
        match GilrsBuilder::new()
            .with_default_filters(false)
            .set_update_state(false)
            .build()
        {
            Ok(gilrs) => {
                app.insert_non_send_resource(InputGilrs::new(gilrs))
                    .init_non_send_resource::<RunningRumbleEffects>()
                    .add_systems(PreStartup, gilrs_event_startup_system)
                    .add_systems(PreUpdate, gilrs_event_system)
                    .add_systems(PostUpdate, play_gilrs_rumble.in_set(RumbleSystem));
            }
            Err(err) => {
                panic!("Failed to start Gilrs. {}", err);
            },
        }
    }
}

pub struct InputGilrs {
    gilrs: Gilrs,

    pressed_gamepad_buttons: HashSet<GamepadButton>,
}

impl InputGilrs {
    pub fn new(gilrs: Gilrs) -> Self {
        Self {
            gilrs,
            pressed_gamepad_buttons: HashSet::new(),
        }
    }

    pub fn press(&mut self, input: GamepadButton) {
        // Returns `true` if the `input` wasn't pressed.
        self.pressed_gamepad_buttons.insert(input);
    }

    pub fn release(&mut self, input: GamepadButton) {
        self.pressed_gamepad_buttons.remove(&input);
    }

    pub fn is_pressed(&self, input: GamepadButton) -> bool {
        self.pressed_gamepad_buttons.contains(&input)
    }

    pub fn reset(&mut self, input: GamepadButton) {
        self.pressed_gamepad_buttons.remove(&input);
    }
}
