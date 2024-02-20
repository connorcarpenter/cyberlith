#[macro_use]
extern crate cfg_if;

mod asset_cache;
mod asset_metadata_store;
mod region_connection;
mod state;
mod asset_endpoint;

cfg_if! {
    if #[cfg(feature = "local")] {
        mod local;
    } else {}
}

use std::{net::SocketAddr, time::Duration};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::{ASSET_SERVER_PORT, SELF_BINDING_ADDR};
use crate::asset_metadata_store::AssetMetadataStore;

use crate::state::State;

pub fn main() {
    // setup logging
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    #[cfg(feature = "local")]
    local::setup();

    // setup state
    let registration_resend_rate = Duration::from_secs(5);
    let region_server_disconnect_timeout = Duration::from_secs(16);
    let asset_path = "assets";
    let asset_metadata_store = AssetMetadataStore::new(asset_path);
    let cache_size_kb = 5000; // 5 MB
    let state = Arc::new(RwLock::new(State::new(
        registration_resend_rate,
        region_server_disconnect_timeout,
        cache_size_kb,
        asset_metadata_store,
        asset_path,
    )));

    // setup listening http server
    info!("Asset Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ASSET_SERVER_PORT);

    let mut server = Server::new(socket_addr);

    region_connection::recv_heartbeat_request(&mut server, state.clone());
    asset_endpoint::handle_asset_request(&mut server, state.clone());

    server.start();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        info!(".");

        // send registration
        let state_clone = state.clone();
        Server::spawn(async move {
            region_connection::send_register_instance_request(state_clone).await;
        });

        // handle disconnection
        let state_clone = state.clone();
        Server::spawn(async move {
            region_connection::process_region_server_disconnect(state_clone).await;
        });
    }
}
