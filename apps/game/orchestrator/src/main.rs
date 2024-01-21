use std::{net::SocketAddr, str::FromStr};

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use http_client::HttpClient;
use http_server::Server;

use orchestrator_proto::{LoginRequest as OrchLoginReq, LoginResponse as OrchLoginRes};
use region_server_proto::LoginRequest as RegLoginReq;

const ADDRESS: &str = "127.0.0.1:14197";

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Orchestrator starting up...");
    let socket_addr: SocketAddr = ADDRESS.parse().unwrap();

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

    let request = RegLoginReq::new(&incoming_request.username, &incoming_request.password);
    let socket_addr = SocketAddr::from_str("127.0.0.1:14198").unwrap();
    let Ok(outgoing_response) = HttpClient::send(&socket_addr, request).await else {
        warn!("Failed login request to region server");
        return Err(());
    };

    info!("Received login response from region server: {}", outgoing_response.token);

    info!("Sending login response to client");

    Ok(OrchLoginRes::new("yeet from orchestrator!"))
}