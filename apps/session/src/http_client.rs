
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;

use bevy_http_client::HttpClient;

use region_server_http_proto::{SessionRegisterInstanceRequest, WorldUserLoginRequest};
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use config::{REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, SESSION_SERVER_RECV_ADDR, SESSION_SERVER_HTTP_PORT, SESSION_SERVER_SIGNAL_PORT, SESSION_SERVER_SECRET, PUBLIC_IP_ADDR};

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
        SESSION_SERVER_SECRET,
        SESSION_SERVER_RECV_ADDR,
        SESSION_SERVER_HTTP_PORT,
        format!("http://{}:{}", PUBLIC_IP_ADDR, SESSION_SERVER_SIGNAL_PORT).as_str(),
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

pub fn send_world_connect_request(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    let worldless_users = global.take_worldless_users();
    for user_key in worldless_users {
        let request = WorldUserLoginRequest::new(SESSION_SERVER_SECRET);
        let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);
        global.add_world_key(&user_key, key);
    }
}

pub fn recv_world_connect_response(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    let mut received_response_keys = Vec::new();
    let mut failed_response_user_keys = Vec::new();
    for (response_key, user_key) in global.world_keys() {
        if let Some(result) = http_client.recv(response_key) {
            received_response_keys.push(response_key.clone());
            match result {
                Ok(response) => {
                    info!("received from regionserver: world_connect(public_url: {:?}, token: {:?})", response.world_server_public_url, response.token);

                    let token = WorldConnectToken::new(
                        &response.world_server_public_url,
                        &response.token,
                    );
                    server.send_message::<PrimaryChannel, WorldConnectToken>(user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                    failed_response_user_keys.push(user_key.clone());
                }
            }
        }
    }
    for response_key in received_response_keys {
        global.remove_world_key(&response_key);
    }
    for user_key in failed_response_user_keys {
        global.add_worldless_user(&user_key);
    }
}