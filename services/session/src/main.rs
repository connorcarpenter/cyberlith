mod asset;
mod http;
mod region;
mod session_instance;
mod social;
mod user;
mod world;

//

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin};

//

use crate::{
    asset::AssetPlugin, http::HttpPlugin, region::RegionPlugin, session_instance::SessionInstance,
    social::SocialPlugin, user::UserPlugin, world::WorldPlugin,
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
        .add_plugins(HttpPlugin::new())
        .add_plugins(RegionPlugin::new(
            registration_resend_rate,
            region_server_disconnect_timeout,
        ))
        .add_plugins(SocialPlugin::new())
        .add_plugins(WorldPlugin::new(world_connect_resend_rate))
        .add_plugins(AssetPlugin::new())
        .add_plugins(UserPlugin::new())
        // Resources
        .insert_resource(SessionInstance::new(&instance_secret))
        // Run App
        .run();
}
