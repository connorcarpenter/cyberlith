use std::net::SocketAddr;

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;
use http_client::HttpClient;

use http_server::Server;

use region_server_http_proto::{LoginRequest as RegLoginReq, LoginResponse as RegLoginRes};
use session_server_http_proto::{LoginRequest as SeshLoginReq, LoginResponse as SeshLoginRes};

const ADDRESS: &str = "127.0.0.1:14198";

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr = ADDRESS.parse().unwrap();

    let mut server = Server::new(socket_addr);
    server.endpoint(login);
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn login(incoming_request: RegLoginReq) -> Result<RegLoginRes, ()> {
    info!("Login request received from orchestrator");

    info!("Sending login request to session server");

    let request = SeshLoginReq::new(&incoming_request.username, &incoming_request.password);
    let socket_addr = "127.0.0.1:14199".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&socket_addr, request).await else {
        warn!("Failed login request to session server");
        return Err(());
    };

    info!(
        "Received login response from session server: {}",
        outgoing_response.token
    );

    info!("Sending login response to orchestrator");

    Ok(RegLoginRes::new("yeet from regionserver!"))
}
