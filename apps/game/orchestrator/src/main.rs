use std::net::SocketAddr;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

const ADDRESS: &str = "127.0.0.1:14197";

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Orchestrator starting up...");
    let socket_addr: SocketAddr = ADDRESS.parse().unwrap();
    http_server::start_server(socket_addr);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}