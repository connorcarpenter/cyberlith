
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;

use bevy_http_client::HttpClient;

use region_server_http_proto::WorldUserLoginRequest;
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use config::{REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, SESSION_SERVER_GLOBAL_SECRET};

use crate::global::Global;

pub fn send_world_connect_request(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    let worldless_users = global.take_worldless_users();
    for user_key in worldless_users {
        let request = WorldUserLoginRequest::new(SESSION_SERVER_GLOBAL_SECRET);
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
                    info!("received from regionserver: world_connect(public_webrtc_url: {:?}, token: {:?})", response.world_server_public_webrtc_url, response.token);

                    let token = WorldConnectToken::new(
                        &response.world_server_public_webrtc_url,
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