#[macro_use]
extern crate cfg_if;


mod heartbeat;
mod state;
mod asset;
mod registration;
mod disconnection;

cfg_if! {
    if #[cfg(feature = "local")] {
        mod local;
    } else {}
}

use std::{net::SocketAddr, time::Duration};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::{Server, async_dup::Arc, smol::lock::RwLock};

use config::{ASSET_SERVER_PORT, SELF_BINDING_ADDR};

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
    let state = Arc::new(RwLock::new(State::new(registration_resend_rate, region_server_disconnect_timeout)));

    // setup listening http server
    info!("Asset Server starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ASSET_SERVER_PORT);

    let mut server = Server::new(socket_addr);

    heartbeat::endpoint(&mut server, state.clone());
    asset::endpoint(&mut server);

    server.start();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        info!(".");

        // send registration
        let state_clone = state.clone();
        Server::spawn(async move {
            registration::handle(state_clone).await;
        });

        // handle disconnection
        let state_clone = state.clone();
        Server::spawn(async move {
            disconnection::handle(state_clone).await;
        });
    }
}