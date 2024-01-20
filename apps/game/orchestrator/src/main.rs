use std::net::SocketAddr;

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use http_client::{header, HttpClient};
use http_server::{Server, Method, Request, Response, ResponseBuilder};

const ADDRESS: &str = "127.0.0.1:14197";

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Orchestrator starting up...");
    let socket_addr: SocketAddr = ADDRESS.parse().unwrap();

    let mut server = Server::new(socket_addr);
    server.endpoint(Method::POST, "login", login);
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn login(incoming_request: Request) -> Result<Response, ()> {
    info!("Login request received");

    let mut outgoing_request = Request::new("".to_string());
    *outgoing_request.method_mut() = Method::POST;
    *outgoing_request.uri_mut() = "http://127.0.0.1:14198/login".parse().unwrap();

    let header_name = header::CONTENT_LENGTH;
    let header_value = "0".parse().unwrap();
    outgoing_request.headers_mut().insert(header_name, header_value);

    info!("Sending login request to region server");

    let Ok(outgoing_response) = HttpClient::send(outgoing_request).await else {
        warn!("Failed login request to region server");
        return Err(());
    };

    info!("Received login response from region server");

    let response = ResponseBuilder::new()
        .status(200)
        .body("".to_string())
        .unwrap();

    Ok(response)
}