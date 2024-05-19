mod asset;
mod user_manager;
mod naia;
mod http_server;
mod session_instance;
mod region;
pub mod social;
pub mod world;

//

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::HttpServerPlugin;

use session_server_http_proto::protocol as http_protocol;
use session_server_naia_proto::protocol as naia_protocol;
use social::SocialManager;

//

use crate::{
    asset::{asset_manager, asset_manager::AssetManager}, world::WorldManager,
    region::RegionManager, session_instance::SessionInstance,
    user_manager::UserManager,
};

fn main() {
    logging::initialize();

    let instance_secret = random::generate_random_string(16);
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(16);
    let world_connect_resend_rate = Duration::from_secs(5);

    // Build App
    App::default()
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(NaiaServerPlugin::new(
            NaiaServerConfig::default(),
            naia_protocol(),
        ))
        .add_plugins(HttpServerPlugin::new(http_protocol()))
        .add_plugins(HttpClientPlugin)
        // Resource
        .insert_resource(UserManager::new())
        .insert_resource(RegionManager::new(
            registration_resend_rate,
            region_server_disconnect_timeout
        ))
        .insert_resource(SocialManager::new())
        .insert_resource(WorldManager::new(world_connect_resend_rate))
        .insert_resource(AssetManager::new())
        .insert_resource(SessionInstance::new(&instance_secret))
        // Startup System
        .add_systems(Startup, naia::init)
        .add_systems(Startup, http_server::start)
        // Receive Server Events
        .add_systems(
            Update,
            (
                naia::auth_events,
                naia::connect_events,
                naia::disconnect_events,
                naia::error_events,
                naia::message_events,

                region::processes::send_register_instance_request,
                region::processes::send_world_connect_requests,
                region::processes::process_region_server_disconnect,

                region::http_endpoints::recv_register_instance_response,
                region::http_endpoints::recv_heartbeat_request,
                region::http_endpoints::recv_login_request,
                region::http_endpoints::recv_world_connect_response,
                region::http_endpoints::recv_connect_social_server_request,
                region::http_endpoints::recv_disconnect_social_server_request,
                region::http_endpoints::recv_connect_asset_server_request,
                region::http_endpoints::recv_disconnect_asset_server_request,

                world::http_endpoints::recv_added_asset_id_request,

                asset_manager::update,
            )
                .in_set(ReceiveEvents),
        )
        // Run App
        .run();
}
