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
use systems::{http_server, naia, user_connection};

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
        .insert_resource(AssetManager::new())
        .init_resource::<UserManager>()
        .init_resource::<LobbyManager>()
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, naia::tick_events_startup)
        .add_systems(Startup, http_server::init)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                naia::tick_events,
                user_connection::recv_world_connect_request,
                region_manager::recv_heartbeat_request,
                region_manager::recv_register_instance_response,
                region_manager::send_register_instance_request,
                region_manager::process_region_server_disconnect,
                asset_manager::update,
            )
                .in_set(ReceiveEvents),
        );

    // Run App
    app.run();
}
