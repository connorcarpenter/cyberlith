
use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, SystemSet};

use gilrs::{Gilrs, GilrsBuilder};

mod gamepad;
pub(crate) use gamepad::{ALL_BUTTON_TYPES, ALL_AXIS_TYPES};
pub use gamepad::{GamepadInfo, Gamepads, GamepadButton, GamepadSettings, GamepadAxis, GamepadId, GamepadButtonType, GamepadAxisType, GamepadRumbleRequest, GamepadRumbleIntensity};

mod axis;
pub use axis::{Axis};

mod gilrs_system;
use gilrs_system::{gilrs_event_startup_system, gilrs_event_system};

mod rumble;
mod converter;

use rumble::{play_gilrs_rumble, RunningRumbleEffects};

/// Plugin that provides gamepad handling to an [`App`].
#[derive(Default)]
pub struct GilrsPlugin;

/// Updates the running gamepad rumble effects.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct RumbleSystem;

impl Plugin for GilrsPlugin {
    fn build(&self, app: &mut App) {

        //
        app
            .add_event::<GamepadRumbleRequest>();

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
}

impl InputGilrs {
    pub fn new(gilrs: Gilrs) -> Self {
        Self {
            gilrs,
        }
    }
}
