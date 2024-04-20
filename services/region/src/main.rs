mod endpoints;
mod instances;
mod state;

use std::{net::SocketAddr, thread, time::Duration};

use config::{REGION_SERVER_PORT, SELF_BINDING_ADDR};
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};
use logging::info;

use crate::state::State;

pub fn main() {
    logging::initialize();

    info!("Region Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), REGION_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::new(Duration::from_secs(16))));
    let server_name = "region_server";

    endpoints::session_register_instance(server_name, &mut server, state.clone());
    endpoints::world_register_instance(server_name, &mut server, state.clone());
    endpoints::asset_register_instance(server_name, &mut server, state.clone());

    endpoints::session_connect(server_name, &mut server, state.clone());
    endpoints::world_connect(server_name, &mut server, state.clone());

    server.start();

    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let mut state = state_clone.write().await;
            state.send_heartbeats().await;
            thread::sleep(Duration::from_secs(5));
        }
    });

    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let mut state = state_clone.write().await;
            state.sync_asset_session_instances().await;
            thread::sleep(Duration::from_secs(5));
        }
    });

    thread::park();

    info!("Shutting down...");
}
