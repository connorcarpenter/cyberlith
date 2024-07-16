#[macro_use]
extern crate cfg_if;

mod file_endpoint;
mod file_metadata_store;
mod state;

cfg_if! {
    if #[cfg(feature = "local")] {
        mod local;
    } else {}
}

use std::{net::SocketAddr, thread};

use config::{
    CONTENT_SERVER_CPU_PRIORITY, CONTENT_SERVER_FILES_PATH, CONTENT_SERVER_PORT, SELF_BINDING_ADDR,
    TOTAL_CPU_PRIORITY,
};
use http_server::{
    async_dup::Arc, executor, executor::smol::lock::RwLock, ApiServer, Method, Server,
};
use logging::info;

use crate::{
    file_endpoint::file_endpoint_handler, file_metadata_store::FileMetadataStore, state::State,
};

pub fn main() {
    logging::initialize();
    executor::setup(CONTENT_SERVER_CPU_PRIORITY, TOTAL_CPU_PRIORITY);

    #[cfg(all(feature = "local", not(feature = "nodeploy")))]
    local::setup();

    // setup state
    let file_metadata_store = FileMetadataStore::new(CONTENT_SERVER_FILES_PATH);

    let cache_size_kb = 5000; // 5 MB
    let state = Arc::new(RwLock::new(State::new(cache_size_kb, file_metadata_store)));

    // setup listening http server
    info!("Content Server starting up...");
    let socket_addr: SocketAddr =
        SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), CONTENT_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let content_server = "content_server";

    for file_name in [
        "launcher.html",
        "launcher.js",
        "launcher_bg.wasm",
        "game.html",
        "game.js",
        "game_bg.wasm",
    ]
    .iter()
    {
        let state = state.clone();
        server.raw_endpoint(
            content_server,
            None,
            None,
            Method::Get,
            file_name,
            move |addr, incoming_req| {
                let state = state.clone();
                let file_name = file_name.to_string();
                async move { file_endpoint_handler(addr, incoming_req, state, file_name).await }
            },
        );
    }

    server.start();

    thread::park();

    info!("Shutting down...");
}
