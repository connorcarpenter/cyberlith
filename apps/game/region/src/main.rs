mod state;
mod instances;
mod endpoints;

use std::net::SocketAddr;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::{Server, async_dup::Arc, smol::lock::RwLock};
use config::REGION_SERVER_ADDR;

use crate::state::State;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr = REGION_SERVER_ADDR.parse().unwrap();

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::default()));

    endpoints::session_register_instance(&mut server, state.clone());
    endpoints::world_register_instance(&mut server, state.clone());
    endpoints::session_user_login(&mut server, state.clone());
    endpoints::world_user_login(&mut server, state.clone());

    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}