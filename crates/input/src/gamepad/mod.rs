
use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};
use bevy_ecs::schedule::{IntoSystemConfigs, SystemSet};

use gilrs::GilrsBuilder;

mod gamepad;
pub(crate) use gamepad::{ALL_BUTTON_TYPES, ALL_AXIS_TYPES};
pub use gamepad::{GamepadInfo, Gamepads, GamepadButton, GamepadSettings, GamepadAxis, GamepadId, GamepadButtonType, GamepadAxisType};

mod axis;
pub use axis::{Axis};

mod gilrs_system;
use gilrs_system::{gilrs_event_startup_system, gilrs_event_system};

mod rumble;
pub use rumble::{RumbleManager, GamepadRumbleIntensity};

mod converter;

use crate::gamepad::gilrs_system::InputGilrs;

/// Plugin that provides gamepad handling to an [`App`].
#[derive(Default)]
pub struct GilrsPlugin;

/// Updates the running gamepad rumble effects.
#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct RumbleSystem;

impl Plugin for GilrsPlugin {
    fn build(&self, app: &mut App) {

        // gilrs
        match GilrsBuilder::new()
            .with_default_filters(false)
            .set_update_state(false)
            .build()
        {
            Ok(gilrs) => {
                app
                    // Resources
                    .insert_non_send_resource(InputGilrs::new(gilrs))
                    .init_resource::<RumbleManager>()
                    // Systems
                    .add_systems(PreStartup, gilrs_event_startup_system)
                    .add_systems(PreUpdate, gilrs_event_system)
                    .add_systems(PostUpdate, RumbleManager::update.in_set(RumbleSystem));
            }
            Err(err) => {
                panic!("Failed to start Gilrs. {}", err);
            },
        }
    }
}
