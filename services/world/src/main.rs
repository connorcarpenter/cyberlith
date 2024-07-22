mod resources;
mod region;
mod http;
mod asset;
mod user;
mod world_instance;

cfg_if::cfg_if!(
    if #[cfg(feature = "odst")] {
        mod odst;
    }
);

use std::time::Duration;

use cfg_if::cfg_if;

use bevy_app::{App, ScheduleRunnerPlugin};

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ServerConfig as NaiaServerConfig,
};

use bevy_http_server::executor;
use config::{TOTAL_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY};
use world_server_naia_proto::protocol as naia_protocol;

use crate::{asset::AssetPlugin, http::HttpPlugin, region::RegionPlugin, resources::lobby_manager::LobbyManager, user::UserPlugin};

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
        .add_plugins(RegionPlugin)
        .add_plugins(HttpPlugin)
        .add_plugins(AssetPlugin)
        .add_plugins(UserPlugin)
        // Resource
        .init_resource::<LobbyManager>();

    // Run App
    app.run();
}
