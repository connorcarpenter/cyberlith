use bevy_ecs::change_detection::ResMut;
use logging::{info, warn};

use naia_bevy_server::Server;

use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;

use region_server_http_proto::WorldConnectRequest;
use session_server_http_proto::{UserAssetIdRequest, UserAssetIdResponse};
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};

use config::{REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR};

use crate::{asset::asset_manager::AssetManager, global::Global};

pub fn send_world_connect_request(mut http_client: ResMut<HttpClient>, mut global: ResMut<Global>) {
    let worldless_users = global.get_users_ready_to_connect_to_world();
    for (user_key, user_id) in worldless_users {
        let request = WorldConnectRequest::new(global.instance_secret(), user_id);
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
                    info!("received from regionserver: world_connect(token: {:?})", response.login_token);

                    // store world instance secret with user key
                    global.user_set_world_connected(
                        &user_key,
                        &response.world_server_instance_secret,
                        response.world_server_user_id,
                    );

                    // send world connect token to user
                    // info!("sending world connect token to user");
                    let token = WorldConnectToken::new(
                        &response.login_token,
                    );
                    server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                }
            }
        }
    }
}

pub fn recv_added_asset_id_request(
    global: ResMut<Global>,
    mut http_server: ResMut<HttpServer>,
    mut naia_server: Server,
    mut http_client: ResMut<HttpClient>,
    mut asset_manager: ResMut<AssetManager>,
) {
    while let Some((_addr, request, response_key)) = http_server.receive::<UserAssetIdRequest>() {
        let world_instance_secret = request.world_instance_secret();

        if !global.world_instance_exists(world_instance_secret) {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let user_id = request.user_id();
        let asset_id = request.asset_id();
        let added = request.added();

        info!(
            "received from worldserver: user_asset_request(user_id: {:?}, asset_id: {:?})",
            user_id, asset_id
        );

        let user_key = global
            .get_user_key_from_world_instance(world_instance_secret, &user_id)
            .unwrap();

        if added {
            asset_manager.load_user_asset(
                &mut naia_server,
                &mut http_client,
                global.get_asset_server_url(),
                user_key,
                asset_id,
            );
        } else {
            asset_manager.unload_user_asset(
                global.get_asset_server_url(),
                user_key,
                asset_id,
            );
        }


        //info!("UserAsset Response sent to world server ..");

        http_server.respond(response_key, Ok(UserAssetIdResponse));
    }

    if asset_manager.has_queued_user_asset_requests() {
        if let Some((asset_server_addr, asset_server_port)) = global.get_asset_server_url() {
            asset_manager.process_queued_user_asset_requests(
                &mut naia_server,
                &mut http_client,
                &asset_server_addr,
                asset_server_port,
            );
        }
    }
}
