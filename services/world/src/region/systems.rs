use bevy_ecs::change_detection::{Res, ResMut};

use bevy_http_client::HttpClient;
use config::{REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, WORLD_SERVER_GLOBAL_SECRET, WORLD_SERVER_HTTP_PORT, WORLD_SERVER_RECV_ADDR};
use logging::info;
use region_server_http_proto::WorldRegisterInstanceRequest;

use crate::{region::RegionManager, resources::{world_instance::WorldInstance, user_manager::UserManager}};

pub fn send_register_instance_request(
    mut http_client: ResMut<HttpClient>,
    world_instance: Res<WorldInstance>,
    mut region_manager: ResMut<RegionManager>,
) {
    if region_manager.connected() {
        return;
    }
    if region_manager.waiting_for_registration_response() {
        return;
    }
    if !region_manager.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = WorldRegisterInstanceRequest::new(
        WORLD_SERVER_GLOBAL_SECRET,
        world_instance.instance_secret(),
        WORLD_SERVER_RECV_ADDR,
        WORLD_SERVER_HTTP_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    region_manager.set_register_instance_response_key(key);
    region_manager.sent_to_region_server();
}

pub fn process_region_server_disconnect(
    mut user_manager: ResMut<UserManager>,
    mut region_manager: ResMut<RegionManager>,
) {
    if region_manager.connected() {
        if region_manager.time_to_disconnect() {
            info!("disconnecting from region server");
            region_manager.disconnect_region_server();
            user_manager.disconnect_region_server();
        }
    }
}
