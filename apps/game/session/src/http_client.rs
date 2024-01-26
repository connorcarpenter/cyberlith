
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;

use bevy_http_client::{HttpClient, ResponseError};

use region_server_http_proto::SessionRegisterInstanceRequest;
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use config::{REGION_SERVER_ADDR, SESSION_SERVER_HTTP_ADDR, SESSION_SERVER_SIGNAL_ADDR};

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
    let request = SessionRegisterInstanceRequest::new(
        SESSION_SERVER_HTTP_ADDR.parse().unwrap(),
        SESSION_SERVER_SIGNAL_ADDR.parse().unwrap(),
    );
    let socket_addr = REGION_SERVER_ADDR.parse().unwrap();
    let key = http_client.send(&socket_addr, request);

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
                    match error {
                        ResponseError::HttpError(error_string) => {
                            warn!("http error: {}", error_string);
                        }
                        ResponseError::SerdeError => {
                            warn!("serde error");
                        }
                        ResponseError::None => {
                            warn!("none error?");
                        }
                    }
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

pub fn recv_world_connect_response(
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