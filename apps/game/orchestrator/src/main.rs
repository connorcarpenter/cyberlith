use std::{net::SocketAddr};

use log::{info, warn, LevelFilter};
use simple_logger::SimpleLogger;

use http_client::HttpClient;
use http_server::Server;
use config::{ORCHESTRATOR_ADDR, REGION_SERVER_ADDR};

use orchestrator_http_proto::{LoginRequest as OrchLoginReq, LoginResponse as OrchLoginRes};
use region_server_http_proto::LoginRequest as RegLoginReq;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Orchestrator starting up...");
    let socket_addr: SocketAddr = ORCHESTRATOR_ADDR.parse().unwrap();

    let mut server = Server::new(socket_addr);
    server.endpoint(login);
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn login(incoming_request: OrchLoginReq) -> Result<OrchLoginRes, ()> {
    info!("Login request received from client");

    info!("Sending login request to region server");

    let region_request = RegLoginReq::new(&incoming_request.username, &incoming_request.password);
    let region_server_addr = REGION_SERVER_ADDR.parse().unwrap();
    let Ok(region_response) = HttpClient::send(&region_server_addr, region_request).await else {
        warn!("Failed login request to region server");
        return Err(());
    };

    info!(
        "Received login response from region server: addr: {:?}, token: {}",
        region_response.session_server_addr,
        region_response.token,
    );

    info!("Sending login response to client");

    Ok(OrchLoginRes::new(
        region_response.session_server_addr,
        region_response.token,
    ))
}
