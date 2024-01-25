
use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use world_server_http_proto::{HeartbeatRequest, HeartbeatResponse, IncomingUserRequest, IncomingUserResponse};
use config::WORLD_SERVER_HTTP_ADDR;

pub fn init(mut server: ResMut<HttpServer>) {
    info!("World HTTP Server starting up");

    let socket_addr = WORLD_SERVER_HTTP_ADDR.parse().expect("could not parse HTTP address/port");
    server.listen(socket_addr);
}

pub fn recv_login_request(mut server: ResMut<HttpServer>) {
    while let Some((addr, request, response_key)) = server.receive::<IncomingUserRequest>() {
        info!("Login request received from {} (regionserver?): Login(secret: {}, token: {})", addr, request.region_secret, request.login_token);

        info!("Sending login response to region server ..");

        server.respond(response_key, IncomingUserResponse);
    }
}

pub fn recv_heartbeat_request(mut server: ResMut<HttpServer>) {
    while let Some((addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        info!("Heartbeat request received from {} (regionserver?): Login(secret: {})", addr, request.region_secret);

        info!("Sending heartbeat response to region server ..");

        server.respond(response_key, HeartbeatResponse);
    }
}