
use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use session_server_http_proto::{HeartbeatRequest, HeartbeatResponse, IncomingUserRequest, IncomingUserResponse};
use config::SESSION_SERVER_HTTP_ADDR;

use crate::global::Global;

pub fn init(mut server: ResMut<HttpServer>) {
    info!("Session HTTP Server starting up");

    let socket_addr = SESSION_SERVER_HTTP_ADDR.parse().expect("could not parse HTTP address/port");
    server.listen(socket_addr);
}

pub fn recv_login_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>
) {
    while let Some((addr, request, response_key)) = server.receive::<IncomingUserRequest>() {
        info!("Login request received from {} (regionserver?): Login(secret: {}, token: {})", addr, request.region_secret, request.login_token);

        global.add_login_token(&request.login_token);

        info!("Sending login response to region server ..");

        server.respond(response_key, Ok(IncomingUserResponse));
    }
}

pub fn recv_heartbeat_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        info!("Heartbeat request received from {}: (secret: {})", addr, request.region_secret);

        // setting last heard
        global.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}