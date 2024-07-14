use bevy_ecs::change_detection::ResMut;

use naia_bevy_server::Server;

use bevy_http_client::{log_util, ApiRequest, ApiResponse, HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::REGION_SERVER_SECRET;
use logging::{info, warn};

use region_server_http_proto::{SessionRegisterInstanceResponse, WorldConnectResponse};
use session_server_http_proto::{
    ConnectAssetServerRequest, ConnectAssetServerResponse, ConnectSocialServerRequest,
    ConnectSocialServerResponse, DisconnectAssetServerRequest, DisconnectAssetServerResponse,
    DisconnectSocialServerRequest, DisconnectSocialServerResponse, HeartbeatRequest,
    HeartbeatResponse, IncomingUserRequest, IncomingUserResponse,
};
use session_server_naia_proto::{channels::PrimaryChannel, messages::WorldConnectToken};

use crate::{
    asset::asset_manager::AssetManager, region::RegionManager, social::SocialManager,
    user::UserManager, world::WorldManager,
};

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    mut region: ResMut<RegionManager>,
) {
    if let Some(response_key) = region.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            let host = "session";
            let remote = "region";
            log_util::recv_res(host, remote, SessionRegisterInstanceResponse::name());
            match result {
                Ok(_response) => {
                    // info!("received from regionserver: instance registered!");
                    region.set_connected();
                }
                Err(error) => {
                    warn!("error: {}", error.to_string());
                }
            }
            region.clear_register_instance_response_key();
        }
    }
}

pub fn recv_heartbeat_request(mut region: ResMut<RegionManager>, mut server: ResMut<HttpServer>) {
    while let Some((_addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        log_util::recv_req(host, remote, HeartbeatResponse::name());

        // info!("Heartbeat request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        log_util::send_res(host, HeartbeatResponse::name());
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}

pub fn recv_login_request(mut user_manager: ResMut<UserManager>, mut server: ResMut<HttpServer>) {
    while let Some((_addr, request, response_key)) = server.receive::<IncomingUserRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        let request_str = format!(
            "{} (token: {})",
            IncomingUserRequest::name(),
            request.login_token
        );
        log_util::recv_req(host, remote, &request_str);

        user_manager.add_login_token(&request.user_id, &request.login_token);

        log_util::send_res(host, IncomingUserResponse::name());
        server.respond(response_key, Ok(IncomingUserResponse));
    }
}

pub fn recv_world_connect_response(
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut world_connections: ResMut<WorldManager>,
) {
    for (response_key, user_key) in world_connections.world_connect_response_keys() {
        if let Some(result) = http_client.recv(&response_key) {
            let host = "session";
            let remote = "region";
            log_util::recv_res(host, remote, WorldConnectResponse::name());

            world_connections.remove_world_connect_response_key(&response_key);
            match result {
                Ok(response) => {
                    info!("(login_token: {:?})", response.login_token);

                    // store world instance secret with user key
                    user_manager.user_set_world_connected(
                        &user_key,
                        &response.world_server_instance_secret,
                    );
                    world_connections.world_set_user_connected(
                        &user_key,
                        &response.world_server_instance_secret,
                        response.world_server_user_id,
                    );

                    // send world connect token to user
                    // info!("sending world connect token to user");
                    let token = WorldConnectToken::new(&response.login_token);
                    server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
                }
                Err(_) => {
                    warn!("error receiving message from region server..");
                }
            }
        }
    }
}

pub fn recv_connect_asset_server_request(
    mut asset_manager: ResMut<AssetManager>,
    mut region: ResMut<RegionManager>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<ConnectAssetServerRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        log_util::recv_req(host, remote, ConnectAssetServerRequest::name());

        // info!("Connect Asset Server request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // store asset server details
        asset_manager.set_asset_server(request.http_addr(), request.http_port());

        // responding
        // info!("Sending connect asset server response to region server ..");
        log_util::send_res(host, ConnectAssetServerResponse::name());
        server.respond(response_key, Ok(ConnectAssetServerResponse));
    }
}

pub fn recv_disconnect_asset_server_request(
    mut asset_manager: ResMut<AssetManager>,
    mut region: ResMut<RegionManager>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) =
        server.receive::<DisconnectAssetServerRequest>()
    {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        log_util::recv_req(host, remote, DisconnectAssetServerRequest::name());

        // info!("Disconnect Asset Server request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // erase asset server details
        asset_manager.clear_asset_server();

        // responding
        // info!("Sending connect asset server response to region server ..");
        log_util::send_res(host, DisconnectAssetServerResponse::name());
        server.respond(response_key, Ok(DisconnectAssetServerResponse));
    }
}

pub fn recv_connect_social_server_request(
    mut social: ResMut<SocialManager>,
    mut region: ResMut<RegionManager>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<ConnectSocialServerRequest>()
    {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        log_util::recv_req(host, remote, ConnectSocialServerRequest::name());

        // info!("Connect Social Server request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // store social server details
        social.set_social_server(request.http_addr(), request.http_port());

        // responding
        // info!("Sending connect social server response to region server ..");
        log_util::send_res(host, ConnectSocialServerResponse::name());
        server.respond(response_key, Ok(ConnectSocialServerResponse));
    }
}

pub fn recv_disconnect_social_server_request(
    mut social: ResMut<SocialManager>,
    mut region: ResMut<RegionManager>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) =
        server.receive::<DisconnectSocialServerRequest>()
    {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "session";
        let remote = "region";
        log_util::recv_req(host, remote, DisconnectSocialServerRequest::name());

        // setting last heard
        region.heard_from_region_server();

        // erase social server details
        social.clear_social_server();

        // responding
        // info!("Sending connect social server response to region server ..");
        log_util::send_res(host, DisconnectSocialServerResponse::name());
        server.respond(response_key, Ok(DisconnectSocialServerResponse));
    }
}
