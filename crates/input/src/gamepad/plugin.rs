use bevy_app::{App, Plugin, PostUpdate, PreStartup, PreUpdate};

use gilrs::GilrsBuilder;

use crate::{gamepad::gilrs::GilrsWrapper, RumbleManager};

#[derive(Default)]
pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
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
                    .insert_non_send_resource(GilrsWrapper::new(gilrs))
                    .init_resource::<RumbleManager>()
                    // Systems
                    .add_systems(PreStartup, GilrsWrapper::startup)
                    .add_systems(PreUpdate, GilrsWrapper::update)
                    .add_systems(PostUpdate, RumbleManager::update);
            }
            Err(err) => {
                panic!("Failed to start Gilrs. {}", err);
            }
        }
    }
}
