use std::net::SocketAddr;
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};
use bevy_http_client::ResponseError;

use bevy_http_server::HttpServer;

use world_server_http_proto::{HeartbeatRequest, HeartbeatResponse, IncomingUserRequest, IncomingUserResponse};
use config::{SELF_BINDING_ADDR, WORLD_SERVER_HTTP_PORT, REGION_SERVER_SECRET};

use crate::global::Global;

pub fn init(mut server: ResMut<HttpServer>) {
    info!("World HTTP Server starting up");

    let socket_addr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_HTTP_PORT);
    server.listen(socket_addr);
}

pub fn recv_login_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
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