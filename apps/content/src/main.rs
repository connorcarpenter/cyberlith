
mod server;

use std::{net::SocketAddr};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use config::CONTENT_SERVER_ADDR;

use crate::server::Server;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Content Server starting up...");
    let socket_addr: SocketAddr = CONTENT_SERVER_ADDR.parse().unwrap();

    let mut server = Server::new(socket_addr);

    server.serve_file("index.html");
    server.serve_file("app.js");
    server.serve_file("app_bg.wasm");

    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}