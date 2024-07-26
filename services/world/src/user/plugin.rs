use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use world_server_naia_proto::protocol as naia_protocol;

use super::systems;
use crate::user::UserManager;

pub struct UserPlugin;

impl Plugin for UserPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(NaiaServerPlugin::new(
                NaiaServerConfig::default(),
                naia_protocol(),
            ))
            // Resources
            .init_resource::<UserManager>()
            // Startup Systems
            .add_systems(
                Startup,
                (systems::startup::server, systems::tick::tick_events_startup),
            )
            // Receive Server Events
            .add_systems(
                Update,
                (
                    systems::connection::auth_events,
                    systems::connection::connect_events,
                    systems::connection::disconnect_events,
                    systems::error::error_events,
                    systems::tick::tick_events,
                )
                    .in_set(ReceiveEvents),
            );
    }
}
