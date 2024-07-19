use std::time::Duration;

use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use super::{http_endpoints, systems};
use crate::region::RegionManager;

pub struct RegionPlugin {
    region_server_disconnect_timeout: Duration,
    registration_resend_rate: Duration,
}

impl RegionPlugin {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            registration_resend_rate,
            region_server_disconnect_timeout,
        }
    }
}

impl Plugin for RegionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RegionManager::new(
            self.registration_resend_rate,
            self.region_server_disconnect_timeout,
        ))
        .add_systems(
            Update,
            (
                systems::send_register_instance_request,
                systems::process_region_server_disconnect,
                http_endpoints::recv_register_instance_response,
                http_endpoints::recv_heartbeat_request,
                http_endpoints::recv_login_request,
                http_endpoints::recv_connect_social_server_request,
                http_endpoints::recv_disconnect_social_server_request,
                http_endpoints::recv_connect_asset_server_request,
                http_endpoints::recv_disconnect_asset_server_request,
            )
                .in_set(ReceiveEvents),
        );
    }
}
