mod naia;
mod http_server;
mod http_client;
mod global;

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_log::LogPlugin;

use naia_bevy_server::{Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig};
use bevy_http_client::HttpClientPlugin;

use bevy_http_server::HttpServerPlugin;

use world_server_naia_proto::{protocol as naia_protocol};
use world_server_http_proto::{protocol as http_protocol};

use global::Global;

fn main() {
    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(LogPlugin::default())
        .add_plugins(NaiaServerPlugin::new(NaiaServerConfig::default(), naia_protocol()))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        .add_plugins(HttpClientPlugin)
        // Resource
        .init_resource::<Global>()
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, http_server::init)
        .add_systems(Startup, http_client::send_register_instance)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                http_server::recv_login_request,
                http_server::recv_heartbeat_request,
                http_client::recv_register_instance_response,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
