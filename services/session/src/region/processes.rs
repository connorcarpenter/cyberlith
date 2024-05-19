use bevy_ecs::{system::Res, change_detection::ResMut};

use bevy_http_client::HttpClient;
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR,
    SESSION_SERVER_GLOBAL_SECRET, SESSION_SERVER_HTTP_PORT, SESSION_SERVER_RECV_ADDR,
};
use logging::info;

use region_server_http_proto::SessionRegisterInstanceRequest;

use crate::{global::Global, region::RegionConnection};

pub fn send_register_instance_request(
    global: Res<Global>,
    mut region: ResMut<RegionConnection>,
    mut http_client: ResMut<HttpClient>,
) {
    if region.connected() {
        return;
    }
    if region.waiting_for_registration_response() {
        return;
    }
    if !region.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = SessionRegisterInstanceRequest::new(
        SESSION_SERVER_GLOBAL_SECRET,
        global.instance_secret(),
        SESSION_SERVER_RECV_ADDR,
        SESSION_SERVER_HTTP_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    region.set_register_instance_response_key(key);
    region.sent_to_region_server();
}

pub fn process_region_server_disconnect(
    mut region: ResMut<RegionConnection>,
) {
    if region.connected() {
        if region.time_to_disconnect() {
            info!("disconnecting from region server");
            region.set_disconnected();
        }
    }
}