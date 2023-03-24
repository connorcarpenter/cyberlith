use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_log::{info, LogPlugin};

use naia_bevy_server::{Plugin as ServerPlugin, ReceiveEvents, ServerConfig};

use cybl_nexus_proto::protocol;

mod resources;
mod systems;

use systems::{events, init};

fn main() {
    info!("Naia Bevy Server Demo starting up");

    // Build App
    App::default()
        // Plugins
        .add_plugin(TaskPoolPlugin::default())
        .add_plugin(TypeRegistrationPlugin::default())
        .add_plugin(FrameCountPlugin::default())
        .insert_resource(
            // this is needed to avoid running the server at uncapped FPS
            ScheduleRunnerSettings::run_loop(Duration::from_millis(5)),
        )
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(ServerPlugin::new(ServerConfig::default(), protocol()))
        // Startup System
        .add_startup_system(init)
        // Receive Server Events
        .add_systems(
            (
                events::auth_events,
                events::connect_events,
                events::disconnect_events,
                events::error_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
