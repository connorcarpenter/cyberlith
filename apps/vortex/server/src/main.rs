use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::{
    schedule::{apply_deferred, IntoSystemConfigs},
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
        changelist_manager_process, ChangelistManager, ShapeManager, ShapeWaitlist, TabManager,
        UserManager,
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
        .add_plugins(ConfigPlugin)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(LogPlugin::default())
        .add_plugins(ServerPlugin::new(server_config, protocol()))
        // Resources
        .init_resource::<UserManager>()
        .init_resource::<GitManager>()
        .init_resource::<TabManager>()
        .init_resource::<ChangelistManager>()
        .init_resource::<ShapeWaitlist>()
        .init_resource::<ShapeManager>()
        // Network Systems
        .add_systems(Startup, network::init)
        .add_systems(
            Update,
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
            Update,
            (
                network::insert_component_events,
                apply_deferred,
                network::message_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        )
        // Other Systems
        .add_systems(Startup, setup)
        .add_systems(Update, world_loop.after(ReceiveEvents))
        .add_systems(Update, changelist_manager_process)
        // Run App
        .run();
}

fn setup(config: Res<AppConfig>, mut git_manager: ResMut<GitManager>) {
    info!("Environment: {}", config.general.env_name);

    git_manager.use_config(&config.git);
}
