use std::collections::HashSet;
use std::net::SocketAddr;

use log::{info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use config::REGION_SERVER_ADDR;
use http_client::HttpClient;
use http_server::{Server, async_dup::Arc, smol::lock::RwLock};
use region_server_http_proto::{
    SessionUserLoginRequest as RegSeshUserLoginReq,
    SessionUserLoginResponse as RegSeshUserLoginRes,
    WorldUserLoginRequest as RegWorldUserLoginReq,
    WorldUserLoginResponse as RegWorldUserLoginRes,
    SessionRegisterInstanceRequest as RegSeshRegisterReq,
    SessionRegisterInstanceResponse as RegSeshRegisterRes,
};
use session_server_http_proto::IncomingUserRequest as SeshIncomingUserReq;
use world_server_http_proto::IncomingUserRequest as WorldIncomingUserReq;

pub struct State {
    session_instances: HashSet<SocketAddr>
}

impl Default for State {
    fn default() -> Self {
        State {
            session_instances: HashSet::new()
        }
    }
}

impl State {
    pub fn register_session_instance(&mut self, addr: SocketAddr) {
        self.session_instances.insert(addr);
    }
}

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .expect("A logger was already initialized");

    info!("Region Server starting up...");
    let socket_addr: SocketAddr = REGION_SERVER_ADDR.parse().unwrap();

    let mut server = Server::new(socket_addr);
    let state = Arc::new(RwLock::new(State::default()));

    let state_1 = state.clone();
    server.endpoint(
        move |(addr, req)| {
            let state_2 = state_1.clone();
            async move {
                let mut state = state_2.write().await;
                state.register_session_instance(addr);
                session_register_instance(req).await
            }
        }
    );
    server.endpoint(
        move |(_addr, req)| {
            async move {
                session_user_login(req).await
            }
        }
    );
    server.endpoint(
        move |(_addr, req)| {
            async move {
                world_user_login(req).await
            }
        }
    );
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn session_register_instance(incoming_request: RegSeshRegisterReq) -> Result<RegSeshRegisterRes, ()> {
    info!("register instance request received from session server");

    info!("Sending register instance response to session server");

    // TODO: end of part we need to get rid of

    Ok(RegSeshRegisterRes)
}

async fn session_user_login(incoming_request: RegSeshUserLoginReq) -> Result<RegSeshUserLoginRes, ()> {
    info!("session user login request received from orchestrator");

    info!("Sending incoming user request to session server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = SeshIncomingUserReq::new(temp_region_secret, temp_token);

    // TODO: this is the part we need to get rid of

    let session_server_http_addr = "127.0.0.1:14199".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&session_server_http_addr, request).await else {
        warn!("Failed incoming user request to session server");
        return Err(());
    };

    info!("Received incoming user response from session server");

    info!("Sending user login response to orchestrator");

    let session_server_signaling_addr = "127.0.0.1:14200".parse().unwrap();

    // TODO: end of part we need to get rid of

    Ok(RegSeshUserLoginRes::new(session_server_signaling_addr, temp_token))
}

async fn world_user_login(incoming_request: RegWorldUserLoginReq) -> Result<RegWorldUserLoginRes, ()> {
    info!("world user login request received from session server");

    info!("sending incoming user request to world server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    // TODO: this is the part we need to get rid of

    let request = WorldIncomingUserReq::new(temp_region_secret, temp_token);
    let world_server_http_addr = "127.0.0.1:14202".parse().unwrap();
    let Ok(outgoing_response) = HttpClient::send(&world_server_http_addr, request).await else {
        warn!("Failed incoming user request to world server");
        return Err(());
    };

    info!("Received incoming user response from world server");

    info!("Sending user login response to session server");

    let world_server_signaling_addr = "127.0.0.1:14203".parse().unwrap();

    // TODO: end of part we need to get rid of

    Ok(RegWorldUserLoginRes::new(world_server_signaling_addr, temp_token))
}
