mod naia;
mod http;

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_log::LogPlugin;

use naia_bevy_server::{Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig};

use bevy_http_server::HttpServerPlugin;

use session_server_naia_proto::{protocol as naia_protocol};
use session_server_http_proto::{protocol as http_protocol};

fn main() {
    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(LogPlugin::default())
        .add_plugins(NaiaServerPlugin::new(NaiaServerConfig::default(), naia_protocol()))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, http::init)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                http::login_recv,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
