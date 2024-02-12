mod heartbeat;

use std::{net::SocketAddr};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::Server;
use config::{ASSET_SERVER_PORT, SELF_BINDING_ADDR};

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Asset Server starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ASSET_SERVER_PORT);

    let mut server = Server::new(socket_addr);

    heartbeat::endpoint(&mut server);

    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}