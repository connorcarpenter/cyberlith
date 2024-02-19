use std::net::SocketAddr;

use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use bevy_http_server::HttpServer;

use config::{SELF_BINDING_ADDR, WORLD_SERVER_HTTP_PORT};

pub fn init(mut server: ResMut<HttpServer>) {
    info!("World HTTP Server starting up");

    let socket_addr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_HTTP_PORT);
    server.listen(socket_addr);
}