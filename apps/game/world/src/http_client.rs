
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use bevy_http_client::HttpClient;

use region_server_http_proto::WorldRegisterInstanceRequest;
use config::{REGION_SERVER_ADDR, WORLD_SERVER_HTTP_ADDR, WORLD_SERVER_SIGNAL_ADDR};

use crate::global::Global;

pub fn send_register_instance(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    info!("Sending request to register instance with region server ..");
    let request = WorldRegisterInstanceRequest::new(
        WORLD_SERVER_HTTP_ADDR.parse().unwrap(),
        WORLD_SERVER_SIGNAL_ADDR.parse().unwrap(),
    );
    let socket_addr = REGION_SERVER_ADDR.parse().unwrap();
    let key = http_client.send(&socket_addr, request);
    global.set_register_instance_response_key(key);
}

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    global: ResMut<Global>,
) {
    if let Some(response_key) = global.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            match result {
                Ok(_response) => {
                    info!("received from regionserver: instance registered!");
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                }
            }
        }
    }
}