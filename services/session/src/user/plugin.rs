
use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ReceiveEvents, ServerConfig as NaiaServerConfig,
};

use session_server_naia_proto::protocol as naia_protocol;

use super::{user_manager::UserManager, systems};

pub struct UserPlugin {

}

impl UserPlugin {
    pub fn new(

    ) -> Self {
        Self {

        }
    }
}

impl Plugin for UserPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NaiaServerPlugin::new(
                NaiaServerConfig::default(),
                naia_protocol(),
            ))
            .insert_resource(UserManager::new())
            .add_systems(Startup, systems::startup::server)
            .add_systems(
                Update,
                (
                    systems::auth_events,
                    systems::connect_events,
                    systems::disconnect_events,
                    systems::error_events,
                    systems::message_events,
                )
                .in_set(ReceiveEvents),
            );
    }
}