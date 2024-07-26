use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use super::{http_endpoints, world_manager::WorldManager};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldManager>().add_systems(
            Update,
            http_endpoints::recv_added_asset_id_request.in_set(ReceiveEvents),
        );
    }
}
