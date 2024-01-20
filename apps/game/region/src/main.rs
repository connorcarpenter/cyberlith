use std::net::SocketAddr;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

use http_server::{Server, Method, Request, Response, ResponseBuilder};

const ADDRESS: &str = "127.0.0.1:14198";

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr = ADDRESS.parse().unwrap();

    let mut server = Server::new(socket_addr);
    server.endpoint(Method::POST, "login", login);
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn login(_request: Request) -> Result<Response, ()> {
    info!("Login request received");
    let response = ResponseBuilder::new()
        .status(200)
        .body("".to_string())
        .unwrap();
    info!("Login response sent");
    Ok(response)
}