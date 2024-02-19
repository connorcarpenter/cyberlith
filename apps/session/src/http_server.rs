use std::net::SocketAddr;

use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use config::{SELF_BINDING_ADDR, SESSION_SERVER_HTTP_PORT};

pub fn init(mut server: ResMut<HttpServer>) {
    info!("Session HTTP Server starting up");

    let socket_addr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), SESSION_SERVER_HTTP_PORT);
    server.listen(socket_addr);
}