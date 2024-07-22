use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use session_server_naia_proto::protocol as naia_protocol;

use super::{systems, user_manager::UserManager};

pub struct UserPlugin;

impl Plugin for UserPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NaiaServerPlugin::new(
            NaiaServerConfig::default(),
            naia_protocol(),
        ))
        .insert_resource(UserManager::new())
        .add_systems(Startup, systems::startup::server)
        .add_systems(
            Update,
            (
                UserManager::update,
                systems::auth_events,
                systems::connect_events,
                systems::disconnect_events,
                systems::error_events,
                systems::message_events,
                systems::scope_checks,
            )
                .in_set(ReceiveEvents),
        );
    }
}
