mod resources;
mod systems;
mod region;
mod http;

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

use bevy_http_server::executor;
use config::{TOTAL_CPU_PRIORITY, WORLD_SERVER_CPU_PRIORITY};
use world_server_naia_proto::protocol as naia_protocol;

use crate::{http::HttpPlugin, region::RegionPlugin, resources::{asset_manager, asset_manager::AssetManager, lobby_manager::LobbyManager, user_manager::UserManager}, systems::user_events};

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
        // Resource
        .init_resource::<AssetManager>()
        .init_resource::<UserManager>()
        .init_resource::<LobbyManager>()
        // Startup System
        .add_systems(Startup, (
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
        .add_systems(Update, asset_manager::update);

    // Run App
    app.run();
}
