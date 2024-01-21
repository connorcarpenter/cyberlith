
use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use session_server_http_proto::{LoginRequest, LoginResponse};

pub fn init(mut server: ResMut<HttpServer>) {
    info!("Session HTTP Server starting up");

    let socket_addr = "127.0.0.1:4199".parse().expect("could not parse HTTP address/port");
    server.listen(socket_addr);
}

pub fn login_recv(mut server: ResMut<HttpServer>) {
    while let Some((addr, request, response_key)) = server.receive::<LoginRequest>() {
        info!("Login request received from .. {}", addr);

        info!("Sending login response to someone ..");

        server.respond(response_key, LoginResponse::new("yeet from session!"));
    }
}