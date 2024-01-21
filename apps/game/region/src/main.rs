use std::net::SocketAddr;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::Server;

use region_server_http_proto::{LoginRequest, LoginResponse};

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

async fn login(_incoming_request: LoginRequest) -> Result<LoginResponse, ()> {
    info!("Login request received");

    info!("Sending login response to orchestrator");

    Ok(LoginResponse::new("yeet from regionserver!"))
}
