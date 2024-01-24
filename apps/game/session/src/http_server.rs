
use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use session_server_http_proto::{IncomingUserRequest, IncomingUserResponse};
use config::SESSION_SERVER_HTTP_ADDR;

pub fn init(mut server: ResMut<HttpServer>) {
    info!("Session HTTP Server starting up");

    let socket_addr = SESSION_SERVER_HTTP_ADDR.parse().expect("could not parse HTTP address/port");
    server.listen(socket_addr);
}

pub fn login_recv(mut server: ResMut<HttpServer>) {
    while let Some((addr, request, response_key)) = server.receive::<IncomingUserRequest>() {
        info!("Login request received from {} (regionserver?): Login(secret: {}, token: {})", addr, request.region_secret, request.login_token);

        info!("Sending login response to region server ..");

        server.respond(response_key, IncomingUserResponse);
    }
}