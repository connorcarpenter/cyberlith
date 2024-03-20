
use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy_ecs::system::Resource;

use gilrs::{Gilrs, GilrsBuilder};

mod converter;
mod gamepad;
mod axis;

mod gilrs_system;
use gilrs_system::{gilrs_event_startup_system, gilrs_event_system};

mod rumble;
use rumble::{play_gilrs_rumble, RunningRumbleEffects};
use crate::gamepad::GamepadButton;

/// Plugin that provides gamepad handling to an [`App`].
#[derive(Default)]
pub struct GilrsPlugin;

/// Updates the running gamepad rumble effects.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct RumbleSystem;

impl Plugin for GilrsPlugin {
    fn build(&self, app: &mut App) {
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
}

impl InputGilrs {
    pub fn new(gilrs: Gilrs) -> Self {
        Self {
            gilrs
        }
    }

    pub fn pressed(&self, button: GamepadButton) -> bool {
        todo!()
    }

    pub fn released(&self) -> bool {
        todo!()
    }

    pub fn reset(&mut self, gamepad_button: GamepadButton) {
        todo!()
    }

    pub fn press(&mut self, button: GamepadButton) {
        todo!()
    }

    pub fn release(&mut self, button: GamepadButton) {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }
}
