use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{
    schedule::IntoSystemConfigs,
    system::{Res, ResMut},
};
use bevy_log::{info, LogPlugin};
use naia_bevy_server::{Plugin as ServerPlugin, ReceiveEvents, ServerConfig};

use vortex_proto::protocol;

mod components;
mod config;
mod resources;
mod systems;

use resources::GitManager;
use systems::network;

use crate::{
    config::{AppConfig, ConfigPlugin},
    resources::UserManager,
};

fn main() {
    info!("Vortex Server starting up");

    let mut server_config = ServerConfig::default();
    server_config.connection.disconnection_timeout_duration = Duration::from_secs(10);

    // Build App
    App::default()
        // Plugins
        .add_plugin(ConfigPlugin)
        .insert_resource(
            // this is needed to avoid running the server at uncapped FPS
            ScheduleRunnerSettings::run_loop(Duration::from_millis(5)),
        )
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(LogPlugin::default())
        .add_plugin(ServerPlugin::new(server_config, protocol()))
        // Resources
        .init_resource::<UserManager>()
        .init_resource::<GitManager>()
        // Network Systems
        .add_startup_system(network::init)
        .add_systems(
            (
                network::auth_events,
                network::connect_events,
                network::disconnect_events,
                network::error_events,
                network::tick_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Other Systems
        .add_startup_system(setup)
        // Run App
        .run();
}

fn setup(config: Res<AppConfig>, mut git_manager: ResMut<GitManager>) {
    info!("Environment: {}", config.general.env_name);

    git_manager.use_config(&config.git);
}
