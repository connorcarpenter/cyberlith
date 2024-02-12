mod heartbeat;
mod state;
mod asset;

use std::{net::SocketAddr, time::Duration};

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use http_server::{Server, async_dup::Arc, smol::lock::RwLock};
use http_client::HttpClient;

use region_server_http_proto::AssetRegisterInstanceRequest;

use config::{ASSET_SERVER_PORT, SELF_BINDING_ADDR, ASSET_SERVER_RECV_ADDR, ASSET_SERVER_SECRET, REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT};

use crate::state::State;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    // setup listening http server
    info!("Asset Server starting up...");
    let socket_addr: SocketAddr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), ASSET_SERVER_PORT);

    let mut server = Server::new(socket_addr);
    let _state = Arc::new(RwLock::new(State::new()));

    heartbeat::endpoint(&mut server);
    asset::endpoint(&mut server);

    server.start();

    // send registration request to region server
    Server::spawn(async move {
        let request = AssetRegisterInstanceRequest::new(
            ASSET_SERVER_SECRET,
            ASSET_SERVER_RECV_ADDR,
            ASSET_SERVER_PORT,
        );
        let response = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
        match response {
            Ok(_) => {
                info!("from {:?}:{} - asset server registration success", REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT);
            },
            Err(err) => {
                warn!("from {:?}:{} - asset server registration failure: {}", REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, err.to_string());
            }
        }
    });

    // loop
    loop {
        std::thread::sleep(Duration::from_secs(1));
        info!(".");
    }
}