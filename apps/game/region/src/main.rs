use std::net::SocketAddr;

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use config::REGION_SERVER_ADDR;
use http_client::HttpClient;
use http_server::Server;
use region_server_http_proto::{SessionConnectRequest as RegSeshConnectReq, SessionConnectResponse as RegSeshConnectRes, WorldConnectRequest as RegWorldConnectReq, WorldConnectResponse as RegWorldConnectRes};
use session_server_http_proto::IncomingUserRequest as SeshIncomingUserReq;
use world_server_http_proto::IncomingUserRequest as WorldIncomingUserReq;

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr = REGION_SERVER_ADDR.parse().unwrap();

    let mut server = Server::new(socket_addr);
    server.endpoint(session_connect);
    server.endpoint(world_connect);
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn session_connect(incoming_request: RegSeshConnectReq) -> Result<RegSeshConnectRes, ()> {
    info!("session connection request received from orchestrator");

    info!("Sending login request to session server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = SeshIncomingUserReq::new(temp_region_secret, temp_token);

    // TODO: this is the part we need to get rid of

    let session_server_http_addr = "127.0.0.1:14199".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&session_server_http_addr, request).await else {
        warn!("Failed login request to session server");
        return Err(());
    };

    info!("Received login response from session server");

    info!("Sending login response to orchestrator");

    let session_server_signaling_addr = "127.0.0.1:14200".parse().unwrap();

    // TODO: end of part we need to get rid of

    Ok(RegSeshConnectRes::new(session_server_signaling_addr, temp_token))
}

async fn world_connect(incoming_request: RegWorldConnectReq) -> Result<RegWorldConnectRes, ()> {
    info!("world connection request received from session server");

    info!("sending login request to world server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    // TODO: this is the part we need to get rid of

    let request = WorldIncomingUserReq::new(temp_region_secret, temp_token);
    let world_server_http_addr = "127.0.0.1:14202".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&world_server_http_addr, request).await else {
        warn!("Failed login request to world server");
        return Err(());
    };

    info!("Received login response from world server");

    info!("Sending login response to session server");

    let world_server_signaling_addr = "127.0.0.1:14203".parse().unwrap();

    // TODO: end of part we need to get rid of

    Ok(RegWorldConnectRes::new(world_server_signaling_addr, temp_token))
}
