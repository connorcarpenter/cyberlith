mod server;

use std::{net::SocketAddr, thread};

use logging::info;

use config::{CONTENT_SERVER_PORT, SELF_BINDING_ADDR};

use crate::server::Server;

pub fn main() {
    logging::initialize();

    info!("Content Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), CONTENT_SERVER_PORT);

    let mut server = Server::new(socket_addr);

    server.serve_file("launcher.html");
    server.serve_file("launcher.js");
    server.serve_file("launcher_bg.wasm");

    server.serve_file("game.html");
    server.serve_file("game.js");
    server.serve_file("game_bg.wasm");

    server.start();

    thread::park();

    info!("Shutting down...");
}
