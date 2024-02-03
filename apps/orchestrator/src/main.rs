mod endpoint;

use std::{net::SocketAddr};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::Server;
use config::{ORCHESTRATOR_PORT, SELF_BINDING_ADDR};

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Orchestrator starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ORCHESTRATOR_PORT);

    let mut server = Server::new(socket_addr);

    endpoint::world_user_login(&mut server);

    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}