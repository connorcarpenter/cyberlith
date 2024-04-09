mod server;

use std::{net::SocketAddr, thread};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use config::{CONTENT_SERVER_PORT, SELF_BINDING_ADDR};

use crate::server::Server;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Content Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), CONTENT_SERVER_PORT);

    let mut server = Server::new(socket_addr);

    server.serve_file_masked("", "index.html");
    server.serve_file("index.html");
    server.serve_file("target/game_client.js");
    server.serve_file("target/game_client_bg.wasm");

    server.start();

    thread::park();

    info!("Shutting down...");
}
