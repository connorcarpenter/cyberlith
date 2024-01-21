mod components;
mod resources;
mod systems;

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_log::LogPlugin;

use naia_bevy_server::{Plugin as ServerPlugin, ReceiveEvents, ServerConfig};

use session_proto::protocol;
use systems::network;



fn main() {

    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(LogPlugin::default())
        .add_plugins(ServerPlugin::new(ServerConfig::default(), protocol()))
        // Startup System
        .add_systems(Startup, network::init)
        // Receive Server Events
        .add_systems(
            Update,
            (
                network::auth_events,
                network::connect_events,
                network::disconnect_events,
                network::error_events,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
