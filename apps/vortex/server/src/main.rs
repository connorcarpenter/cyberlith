use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{schedule::IntoSystemConfigs, system::ResMut};
use bevy_log::{info, LogPlugin};

use naia_bevy_server::{Plugin as ServerPlugin, ReceiveEvents, ServerConfig};

use vortex_proto::protocol;

mod components;
mod resources;
mod systems;

use systems::network;
use resources::GitManager;

fn main() {
    info!("Naia Bevy Server Demo starting up");

    // Build App
    App::default()
        // Plugins
        .insert_resource(
            // this is needed to avoid running the server at uncapped FPS
            ScheduleRunnerSettings::run_loop(Duration::from_millis(5)),
        )
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(ServerPlugin::new(ServerConfig::default(), protocol()))
        // Network Systems
        .add_startup_system(network::init)
        .add_systems(
            (
                network::auth_events,
                network::connect_events,
                network::disconnect_events,
                network::error_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Other Systems
        .insert_resource(GitManager::default())
        .add_startup_system(setup)
        // Run App
        .run();
}

fn setup(mut git_manager: ResMut<GitManager>) {
    git_manager.init();
}
