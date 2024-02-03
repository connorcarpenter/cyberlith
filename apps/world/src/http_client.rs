
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use bevy_http_client::HttpClient;

use region_server_http_proto::WorldRegisterInstanceRequest;
use config::{REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, WORLD_SERVER_RECV_ADDR, WORLD_SERVER_HTTP_PORT, WORLD_SERVER_SIGNAL_PORT, WORLD_SERVER_SECRET};

use crate::global::Global;

pub fn send_connect_region(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    if global.connected() {
        return;
    }
    if global.waiting_for_registration_response() {
        return;
    }
    if !global.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = WorldRegisterInstanceRequest::new(
        WORLD_SERVER_SECRET,
        WORLD_SERVER_RECV_ADDR,
        WORLD_SERVER_HTTP_PORT,
        WORLD_SERVER_RECV_ADDR,
        WORLD_SERVER_SIGNAL_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    global.set_register_instance_response_key(key);
    global.sent_to_region_server();
}

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    if let Some(response_key) = global.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            match result {
                Ok(_response) => {
                    info!("received from regionserver: instance registered!");
                    global.set_connected();
                }
                Err(error) => {
                    warn!("error: {}", error.to_string());
                }
            }
            global.clear_register_instance_response_key();
        }
    }
}

pub fn process_region_server_disconnect(mut global: ResMut<Global>) {
    if global.connected() {
        if global.time_to_disconnect() {
            info!("disconnecting from region server");
            global.set_disconnected();
        }
    }
}