
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;
use bevy_http_client::HttpClient;

use region_server_http_proto::SessionRegisterInstanceRequest;
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use config::REGION_SERVER_ADDR;

use crate::global::Global;

pub fn register_instance_send(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    info!("Sending request to register instance with region server ..");
    let request = SessionRegisterInstanceRequest::new();
    let socket_addr = REGION_SERVER_ADDR.parse().unwrap();
    let key = http_client.send(&socket_addr, request);
    global.set_register_insance_response_key(key);
}

pub fn register_instance_recv(
    mut http_client: ResMut<HttpClient>,
    global: ResMut<Global>,
) {
    if let Some(response_key) = global.register_insance_response_key() {
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

pub fn world_connect_recv(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    let mut received_response_keys = Vec::new();
    for (response_key, user_key) in global.world_keys() {
        if let Some(result) = http_client.recv(response_key) {
            received_response_keys.push(response_key.clone());
            match result {
                Ok(response) => {
                    info!("received from regionserver: world_connect(addr: {:?}, token: {:?})", response.world_server_addr, response.token);

                    let token = WorldConnectToken::new(
                        response.world_server_addr.inner(),
                        &response.token,
                    );
                    server.send_message::<PrimaryChannel, WorldConnectToken>(user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                }
            }
        }
    }
    for response_key in received_response_keys {
        global.remove_world_key(&response_key);
    }
}