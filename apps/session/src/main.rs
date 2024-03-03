//

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_log::LogPlugin;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::HttpServerPlugin;

use session_server_http_proto::protocol as http_protocol;
use session_server_naia_proto::protocol as naia_protocol;

//

mod asset;
mod global;
mod http_server;
mod naia;
mod region_connection;
mod user_connection;
mod world_connection;

use crate::{
    asset::{asset_connection, asset_manager, asset_manager::AssetManager},
    global::Global,
};

fn main() {
    let instance_secret = crypto::generate_random_string(16);
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(16);
    let world_connect_resend_rate = Duration::from_secs(5);

    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(LogPlugin::default())
        .add_plugins(NaiaServerPlugin::new(
            NaiaServerConfig::default(),
            naia_protocol(),
        ))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        .add_plugins(HttpClientPlugin)
        // Resource
        .insert_resource(Global::new(
            &instance_secret,
            registration_resend_rate,
            region_server_disconnect_timeout,
            world_connect_resend_rate,
        ))
        .insert_resource(AssetManager::new())
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, http_server::init)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                user_connection::recv_login_request,
                region_connection::send_register_instance_request,
                region_connection::recv_register_instance_response,
                region_connection::recv_heartbeat_request,
                region_connection::process_region_server_disconnect,
                asset_connection::recv_connect_asset_server_request,
                asset_connection::recv_disconnect_asset_server_request,
                world_connection::send_world_connect_request,
                world_connection::recv_world_connect_response,
                world_connection::recv_added_asset_id_request,
                asset_manager::update,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
