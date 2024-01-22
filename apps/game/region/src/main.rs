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

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = SeshLoginReq::new(temp_region_secret, temp_token);
    let session_server_http_addr = "127.0.0.1:14199".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&session_server_http_addr, request).await else {
        warn!("Failed login request to session server");
        return Err(());
    };

    info!("Received login response from session server");

    info!("Sending login response to orchestrator");

    let session_server_signaling_addr = "127.0.0.1:14200".parse().unwrap();

    Ok(RegLoginRes::new(session_server_signaling_addr, temp_token))
}
