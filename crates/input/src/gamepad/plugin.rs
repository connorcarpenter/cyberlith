
use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};

use gilrs::GilrsBuilder;

use crate::{gamepad::gilrs_system::{gilrs_event_startup_system, gilrs_event_system, InputGilrs}, RumbleManager};

/// Plugin that provides gamepad handling to an [`App`].
#[derive(Default)]
pub struct GilrsPlugin;

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
                    .add_systems(PostUpdate, RumbleManager::update);
            }
            Err(err) => {
                panic!("Failed to start Gilrs. {}", err);
            },
        }
    }
}
