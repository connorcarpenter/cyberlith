mod asset;
mod http;
mod region;
mod social;
mod user;
mod world_instance;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod odst;
    }
);

use std::time::Duration;

use bevy_app::{App, ScheduleRunnerPlugin};

use bevy_http_server::executor;
use config::{TOTAL_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY};

use crate::{
    asset::AssetPlugin, http::HttpPlugin, region::RegionPlugin, social::SocialPlugin,
    user::UserPlugin, world_instance::WorldInstance,
};

fn main() {
    logging::initialize();
    executor::setup(WORLD_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    // Build App
    let mut app = App::default();

    #[cfg(feature = "odst")]
    app.add_plugins(odst::OdstPlugin);

    // WorldInstance
    #[cfg(not(feature = "odst"))]
    let instance_secret = random::generate_random_string(16);

    #[cfg(feature = "odst")]
    let instance_secret = "odst".to_string();
    app.insert_resource(WorldInstance::new(&instance_secret));

    app
        // Plugins
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(5)))
        .add_plugins(RegionPlugin)
        .add_plugins(HttpPlugin)
        .add_plugins(AssetPlugin)
        .add_plugins(UserPlugin)
        .add_plugins(SocialPlugin)
        // Run!
        .run();
}
