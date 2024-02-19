
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use naia_bevy_server::Server;

use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;

use region_server_http_proto::WorldUserLoginRequest;
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};
use config::{REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT};
use session_server_http_proto::{UserAssetIdRequest, UserAssetIdResponse};

use crate::global::Global;

pub fn send_world_connect_request(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    let worldless_users = global.take_worldless_users();
    for user_key in worldless_users {
        let request = WorldUserLoginRequest::new(global.instance_secret());
        let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);
        global.add_world_connect_response_key(&user_key, key);
    }
}

pub fn recv_world_connect_response(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    for (response_key, user_key) in global.world_connect_response_keys() {
        if let Some(result) = http_client.recv(&response_key) {
            global.remove_world_connect_response_key(&response_key);
            match result {
                Ok(response) => {
                    info!("received from regionserver: world_connect(public_webrtc_url: {:?}, token: {:?})", response.world_server_public_webrtc_url, response.login_token);

                    // store world instance secret with user key
                    global.add_worldfull_user(&user_key, &response.world_server_instance_secret, response.world_server_user_id);

                    // send world connect token to user
                    let token = WorldConnectToken::new(
                        &response.world_server_public_webrtc_url,
                        &response.login_token,
                    );
                    server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                    global.add_worldless_user(&user_key);
                }
            }
        }
    }
}

pub fn recv_added_asset_id_request(
    global: ResMut<Global>,
    mut server: ResMut<HttpServer>
) {
    while let Some((_addr, request, response_key)) = server.receive::<UserAssetIdRequest>() {

        if !global.world_instance_exists(request.world_instance_secret()) {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("UserAssetId request received from world server: (user_id: {:?}, asset_id: {:?})", request.user_id(), request.asset_id());

        // todo !

        info!("UserAssetId response to world server ..");

        server.respond(response_key, Ok(UserAssetIdResponse));
    }
}