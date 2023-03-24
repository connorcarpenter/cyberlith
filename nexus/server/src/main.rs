use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_core::{FrameCountPlugin, TaskPoolPlugin, TypeRegistrationPlugin};
use bevy_log::{info, LogPlugin};

use naia_bevy_server::{Plugin as ServerPlugin, ServerConfig};

use cybl_nexus_proto::protocol;

mod resources;
mod systems;

use systems::init;

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
        // Run App
        .run();
}
