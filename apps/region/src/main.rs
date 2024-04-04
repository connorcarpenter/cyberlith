mod endpoints;
mod instances;
mod state;

use std::{net::SocketAddr, time::Duration};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use config::{REGION_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use crate::state::State;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), REGION_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::new(Duration::from_secs(16))));

    endpoints::session_register_instance(&mut server, state.clone());
    endpoints::world_register_instance(&mut server, state.clone());
    endpoints::asset_register_instance(&mut server, state.clone());

    endpoints::session_connect(&mut server, state.clone());
    endpoints::world_connect(&mut server, state.clone());

    server.start();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        info!(".");

        let state_clone = state.clone();
        Server::spawn(async move {
            let mut state = state_clone.write().await;
            state.send_heartbeats().await;
        });

        let state_clone = state.clone();
        Server::spawn(async move {
            let mut state = state_clone.write().await;
            state.sync_asset_session_instances().await;
        });
    }
}
