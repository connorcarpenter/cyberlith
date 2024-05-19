use std::time::Duration;

use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use super::{world_manager::WorldManager, http_endpoints};

pub struct WorldPlugin {
    world_connect_resend_rate: Duration
}

impl WorldPlugin {
    pub fn new(
        world_connect_resend_rate: Duration
    ) -> Self {
        Self {
            world_connect_resend_rate
        }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldManager::new(self.world_connect_resend_rate))
            .add_systems(
                Update,
                http_endpoints::recv_added_asset_id_request.in_set(ReceiveEvents),
            );
    }
}