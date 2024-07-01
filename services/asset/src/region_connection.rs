use std::time::{Duration, Instant};

use http_client::{HttpClient, ResponseError};
use http_server::{
    async_dup::Arc, executor::smol::lock::RwLock, ApiRequest, ApiResponse, ApiServer, Server,
};
use logging::{info, warn};

use asset_server_http_proto::{HeartbeatRequest, HeartbeatResponse};
use region_server_http_proto::{AssetRegisterInstanceRequest, AssetRegisterInstanceResponse};

use config::{
    ASSET_SERVER_GLOBAL_SECRET, ASSET_SERVER_PORT, ASSET_SERVER_RECV_ADDR, REGION_SERVER_PORT,
    REGION_SERVER_RECV_ADDR,
};

use crate::state::State;

pub enum ConnectionState {
    Disconnected,
    Connected,
}

pub struct RegionServerState {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    registration_resend_rate: Duration,
    region_server_disconnect_timeout: Duration,
}

impl RegionServerState {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            registration_resend_rate,
            region_server_disconnect_timeout,
        }
    }

    pub fn time_to_resend_registration(&self) -> bool {
        let time_since_last_sent = self.region_server_last_sent.elapsed();
        time_since_last_sent >= self.registration_resend_rate
    }

    pub fn time_to_disconnect(&self) -> bool {
        let time_since_last_heard = self.region_server_last_heard.elapsed();
        time_since_last_heard >= self.region_server_disconnect_timeout
    }

    pub fn heard_from_region_server(&mut self) {
        self.region_server_last_heard = Instant::now();
    }

    pub fn sent_to_region_server(&mut self) {
        self.region_server_last_sent = Instant::now();
    }

    pub fn connected(&self) -> bool {
        match self.region_server_connection_state {
            ConnectionState::Connected => true,
            ConnectionState::Disconnected => false,
        }
    }

    pub fn set_connected(&mut self) {
        self.region_server_connection_state = ConnectionState::Connected;
        self.heard_from_region_server();
    }

    pub fn set_disconnected(&mut self) {
        self.region_server_connection_state = ConnectionState::Disconnected;
    }
}

pub async fn send_register_instance_request(state: Arc<RwLock<State>>) {
    let state = &mut state.write().await.region_server;

    if state.connected() {
        return;
    }
    if !state.time_to_resend_registration() {
        return;
    }

    let request = AssetRegisterInstanceRequest::new(
        ASSET_SERVER_GLOBAL_SECRET,
        ASSET_SERVER_RECV_ADDR,
        ASSET_SERVER_PORT,
    );

    let host = "asset";
    let remote = "region";
    http_server::log_util::send_req(host, remote, AssetRegisterInstanceRequest::name());
    let response = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
    http_server::log_util::recv_res(host, remote, AssetRegisterInstanceResponse::name());

    match response {
        Ok(_) => {
            // info!(
            //     "from {:?}:{} - asset server registration success",
            //     REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT
            // );
            state.set_connected();
        }
        Err(err) => {
            warn!(
                "from {:?}:{} - asset server registration failure: {}",
                REGION_SERVER_RECV_ADDR,
                REGION_SERVER_PORT,
                err.to_string()
            );
        }
    }

    state.sent_to_region_server();
}

pub async fn process_region_server_disconnect(state: Arc<RwLock<State>>) {
    let state = &mut state.write().await.region_server;

    if state.connected() {
        if state.time_to_disconnect() {
            info!("disconnecting from region server");
            state.set_disconnected();
        }
    }
}

pub fn recv_heartbeat_request(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_heartbeat_request_impl(state, req).await }
    });
}

async fn async_recv_heartbeat_request_impl(
    state: Arc<RwLock<State>>,
    _: HeartbeatRequest,
) -> Result<HeartbeatResponse, ResponseError> {
    // info!("Heartbeat request received from region server, sending response");
    let state = &mut state.write().await.region_server;
    state.heard_from_region_server();
    Ok(HeartbeatResponse)
}
