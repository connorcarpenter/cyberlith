use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::schedule::IntoSystemConfigs;

use naia_bevy_server::{
    ReceiveEvents, Plugin as NaiaServerPlugin, ServerConfig as NaiaServerConfig,
};

use world_server_naia_proto::protocol as naia_protocol;

use crate::user::UserManager;
use super::systems;

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
            .add_systems(Startup, (
                systems::user_events::init,
                systems::user_events::tick_events_startup,
            ))
            // Receive Server Events
            .add_systems(
                Update,
                (
                    systems::user_events::auth_events,
                    systems::user_events::connect_events,
                    systems::user_events::disconnect_events,
                    systems::user_events::error_events,
                    systems::user_events::tick_events,
                )
                    .in_set(ReceiveEvents),
            );
    }
}