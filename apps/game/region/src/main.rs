use std::collections::HashMap;
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

pub struct SessionInstance {
    http_addr: SocketAddr,
    signal_addr: SocketAddr,
}

impl SessionInstance {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr,
            signal_addr,
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr
    }
}

pub struct State {
    session_instances: HashMap<SocketAddr, SessionInstance>,
}

impl Default for State {
    fn default() -> Self {
        State {
            session_instances: HashMap::new()
        }
    }
}

impl State {
    pub fn register_session_instance(&mut self, incoming_addr: SocketAddr, instance: SessionInstance) {
        self.session_instances.insert(incoming_addr, instance);
    }

    pub fn get_available_session_server(&self) -> &SessionInstance {
        self.session_instances.values().next().unwrap()
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

    {
        let state_1 = state.clone();
        server.endpoint(
            move |(addr, req)| {
                let state_2 = state_1.clone();
                async move {
                    session_register_instance(addr, state_2, req).await
                }
            }
        );
    }

    {
        let state_1 = state.clone();
        server.endpoint(
            move |(_addr, req)| {
                let state_2 = state_1.clone();
                async move {
                    session_user_login(state_2, req).await
                }
            }
        );
    }

    {
        server.endpoint(
            move |(_addr, req)| {
                async move {
                    world_user_login(req).await
                }
            }
        );
    }

    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        info!(".");
    }
}

async fn session_register_instance(
    incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: RegSeshRegisterReq
) -> Result<RegSeshRegisterRes, ()> {

    let http_addr = incoming_request.http_addr();
    let signal_addr = incoming_request.signal_addr();

    info!(
        "register instance request received from session server: (incoming: {:?}, http: {:?}, signal: {:?})",
        incoming_addr, http_addr, signal_addr
    );

    let server_instance = SessionInstance::new(http_addr, signal_addr);

    let mut state = state.write().await;
    state.register_session_instance(incoming_addr, server_instance);

    info!("Sending register instance response to session server");

    Ok(RegSeshRegisterRes)
}

async fn session_user_login(
    state: Arc<RwLock<State>>,
    incoming_request: RegSeshUserLoginReq
) -> Result<RegSeshUserLoginRes, ()> {
    info!("session user login request received from orchestrator");

    let state = state.read().await;
    let session_server = state.get_available_session_server();
    let session_server_http_addr = session_server.http_addr();
    let session_server_signaling_addr = session_server.signal_addr();

    info!("Sending incoming user request to session server");

    let temp_region_secret = "the_region_secret";
    let temp_token = "the_login_token";

    let request = SeshIncomingUserReq::new(temp_region_secret, temp_token);

    // TODO: this is the part we need to get rid of

    let Ok(outgoing_response) = HttpClient::send(&session_server_http_addr, request).await else {
        warn!("Failed incoming user request to session server");
        return Err(());
    };

    info!("Received incoming user response from session server");

    info!("Sending user login response to orchestrator");

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
