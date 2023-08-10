use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::{
    prelude::apply_system_buffers,
    schedule::{IntoSystemConfigs, IntoSystemConfig},
    system::{Res, ResMut},
};
use bevy_log::{info, LogPlugin};

use naia_bevy_server::{Plugin as ServerPlugin, ReceiveEvents, ServerConfig};

use resources::GitManager;
use systems::network;
use vortex_proto::protocol;

use crate::{
    config::{AppConfig, ConfigPlugin},
    resources::{
        changelist_manager_process, ChangelistManager, TabManager, UserManager, VertexManager, ShapeWaitlist,
    },
    systems::world_loop,
};

mod components;
mod config;
mod files;
mod resources;
mod systems;

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
        .init_resource::<TabManager>()
        .init_resource::<ChangelistManager>()
        .init_resource::<ShapeWaitlist>()
        .init_resource::<VertexManager>()
        // Network Systems
        .add_startup_system(network::init)
        .add_systems(
            (
                network::auth_events,
                network::connect_events,
                network::disconnect_events,
                network::error_events,
                network::tick_events,
                network::publish_entity_events,
                network::unpublish_entity_events,
                network::spawn_entity_events,
                network::despawn_entity_events,
                network::remove_component_events,
                network::update_component_events,
            )
                .in_set(ReceiveEvents),
        )
        .add_systems(
            (
                network::insert_component_events,
                apply_system_buffers,
                network::message_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Other Systems
        .add_startup_system(setup)
        .add_system(world_loop.after(ReceiveEvents))
        .add_system(changelist_manager_process)
        // Run App
        .run();
}

fn setup(config: Res<AppConfig>, mut git_manager: ResMut<GitManager>) {
    info!("Environment: {}", config.general.env_name);

    git_manager.use_config(&config.git);
}
