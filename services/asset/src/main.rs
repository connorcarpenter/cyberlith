#[macro_use]
extern crate cfg_if;

mod asset_endpoint;
mod asset_metadata_store;
mod region_connection;
mod state;

cfg_if! {
    if #[cfg(feature = "local")] {
        mod local;
    } else {}
}

use std::{net::SocketAddr, thread, time::Duration};

use config::{ASSET_SERVER_CPU_PRIORITY, ASSET_SERVER_FILES_PATH, ASSET_SERVER_PORT, SELF_BINDING_ADDR, TOTAL_CPU_PRIORITY};
use http_server::{async_dup::Arc, executor, executor::smol::{lock::RwLock, Timer}, Server};
use logging::info;

use crate::{asset_metadata_store::AssetMetadataStore, state::State};

pub fn main() {
    logging::initialize();
    executor::setup(ASSET_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    #[cfg(feature = "local")]
    local::setup();

    // setup state
    let asset_metadata_store = AssetMetadataStore::new(ASSET_SERVER_FILES_PATH);

    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(61);
    let cache_size_kb = 5000; // 5 MB
    let state = Arc::new(RwLock::new(State::new(
        registration_resend_rate,
        region_server_disconnect_timeout,
        cache_size_kb,
        asset_metadata_store,
    )));

    // setup listening http server
    info!("Asset Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ASSET_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let host = "asset";

    region_connection::recv_heartbeat_request(host, &mut server, state.clone());
    asset_endpoint::handle_asset_request(host, &mut server, state.clone());

    server.start();

    // send registration
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            region_connection::send_register_instance_request(state_clone_2).await;
            Timer::after(Duration::from_secs(5)).await;
        }
    });

    // handle disconnection
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            region_connection::process_region_server_disconnect(state_clone_2).await;
            Timer::after(Duration::from_secs(5)).await;
        }
    });

    thread::park();

    info!("Shutting down...");
}
