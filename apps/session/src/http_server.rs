use std::net::SocketAddr;

use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;

use session_server_http_proto::{ConnectAssetServerRequest, ConnectAssetServerResponse, DisconnectAssetServerRequest, DisconnectAssetServerResponse, HeartbeatRequest, HeartbeatResponse, IncomingUserRequest, IncomingUserResponse};
use config::{SELF_BINDING_ADDR, SESSION_SERVER_HTTP_PORT, REGION_SERVER_SECRET};

use crate::global::Global;

pub fn init(mut server: ResMut<HttpServer>) {
    info!("Session HTTP Server starting up");

    let socket_addr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), SESSION_SERVER_HTTP_PORT);
    server.listen(socket_addr);
}

pub fn recv_login_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>
) {
    while let Some((_addr, request, response_key)) = server.receive::<IncomingUserRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Login request received from region server: Login(token: {})", request.login_token);

        global.add_login_token(&request.login_token);

        info!("Sending login response to region server ..");

        server.respond(response_key, Ok(IncomingUserResponse));
    }
}

pub fn recv_heartbeat_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<HeartbeatRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Heartbeat request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}

pub fn recv_connect_asset_server_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<ConnectAssetServerRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Connect Asset Server request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // store asset server details
        global.set_asset_server(request.http_addr(), request.http_port());

        // responding
        // info!("Sending connect asset server response to region server ..");
        server.respond(response_key, Ok(ConnectAssetServerResponse));
    }
}

pub fn recv_disconnect_asset_server_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<DisconnectAssetServerRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Disconnect Asset Server request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // erase asset server details
        global.clear_asset_server();

        // responding
        // info!("Sending connect asset server response to region server ..");
        server.respond(response_key, Ok(DisconnectAssetServerResponse));
    }
}