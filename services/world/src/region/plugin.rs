use std::time::Duration;

use bevy_app::{App, Plugin, Update};

use crate::region::{RegionManager, http_endpoints, systems};

pub struct RegionPlugin;

impl Plugin for RegionPlugin {
    fn build(&self, app: &mut App) {

        let registration_resend_rate = Duration::from_secs(5);
        let region_server_disconnect_timeout = Duration::from_secs(61);
        let region_manager = RegionManager::new(
            registration_resend_rate,
            region_server_disconnect_timeout,
        );

        app
            .insert_resource(region_manager)
            .add_systems(
                Update,
            (
                    http_endpoints::recv_heartbeat_request,
                    http_endpoints::recv_register_instance_response,
                    systems::send_register_instance_request,
                    systems::process_region_server_disconnect,
                )
            );
    }
}