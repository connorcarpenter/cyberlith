mod asset_instance;
mod endpoints;
mod requests;
mod session_instance;
mod social_instance;
mod state;
mod world_instance;

use std::{net::SocketAddr, thread, time::Duration};

use config::{
    REGION_SERVER_CPU_PRIORITY, REGION_SERVER_PORT, SELF_BINDING_ADDR, TOTAL_CPU_PRIORITY,
};
use http_server::{
    async_dup::Arc,
    executor,
    executor::smol::{lock::RwLock, Timer},
    Server,
};
use logging::info;

use crate::state::State;

pub fn main() {
    logging::initialize();
    executor::setup(REGION_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    info!("Region Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), REGION_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::new(Duration::from_secs(61))));
    let host = "region";

    endpoints::session_register_instance(host, &mut server, state.clone());
    endpoints::world_register_instance(host, &mut server, state.clone());
    endpoints::asset_register_instance(host, &mut server, state.clone());
    endpoints::social_register_instance(host, &mut server, state.clone());

    endpoints::session_connect(host, &mut server, state.clone());
    endpoints::world_connect(host, &mut server, state.clone());

    server.start();

    let state_clone = state.clone();
    let state_clone2 = state.clone();

    Server::spawn(async move {
        loop {
            let mut state = state_clone.write().await;
            state.send_heartbeats().await;
            Timer::after(Duration::from_secs(5)).await;
        }
    });

    Server::spawn(async move {
        loop {
            let mut state = state_clone2.write().await;
            state.sync_asset_session_instances().await;
            state.sync_social_session_instances().await;
            Timer::after(Duration::from_secs(1)).await;
        }
    });

    thread::park();

    info!("Shutting down...");
}
