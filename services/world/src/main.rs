mod resources;
mod systems;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod odst;
    }
);

use std::time::Duration;

use cfg_if::cfg_if;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::{executor, HttpServerPlugin};
use config::{TOTAL_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY};
use world_server_http_proto::protocol as http_protocol;
use world_server_naia_proto::protocol as naia_protocol;

use resources::{asset_manager::AssetManager, asset_manager, region_manager, user_manager::UserManager};
use systems::{http_server, user_events};

use crate::resources::lobby_manager::LobbyManager;

fn main() {
    logging::initialize();
    executor::setup(WORLD_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    // Build App
    let mut app = App::default();

    cfg_if! {
        if #[cfg(feature = "odst")] {
            app.add_plugins(odst::OdstPlugin);
        }
    }

    app
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(NaiaServerPlugin::new(
            NaiaServerConfig::default(),
            naia_protocol(),
        ))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        .add_plugins(HttpClientPlugin)
        // Resource
        .init_resource::<AssetManager>()
        .init_resource::<UserManager>()
        .init_resource::<LobbyManager>()
        // Startup System
        .add_systems(Startup, (
            http_server::init,
            user_events::init,
            user_events::tick_events_startup,
        ))
        // Receive Server Events
        .add_systems(
            Update,
            (
                user_events::auth_events,
                user_events::connect_events,
                user_events::disconnect_events,
                user_events::error_events,
                user_events::tick_events,
            )
                .in_set(ReceiveEvents),
        )
        .add_systems(
            Update,
            (
                region_manager::recv_heartbeat_request,
                region_manager::recv_register_instance_response,
                region_manager::send_register_instance_request,
                region_manager::process_region_server_disconnect,

                http_server::recv_world_connect_request,

                asset_manager::update,
            )
        );

    // Run App
    app.run();
}
