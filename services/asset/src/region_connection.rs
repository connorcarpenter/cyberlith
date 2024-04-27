use asset_server_http_proto::{HeartbeatRequest, HeartbeatResponse};
use logging::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, smol::lock::RwLock, ApiServer, Server};

use region_server_http_proto::AssetRegisterInstanceRequest;

use config::{
    ASSET_SERVER_GLOBAL_SECRET, ASSET_SERVER_PORT, ASSET_SERVER_RECV_ADDR, REGION_SERVER_PORT,
    REGION_SERVER_RECV_ADDR,
};

use crate::state::State;

pub async fn send_register_instance_request(state: Arc<RwLock<State>>) {
    let mut state = state.write().await;

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
    let response = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
    match response {
        Ok(_) => {
            info!(
                "from {:?}:{} - asset server registration success",
                REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT
            );
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
    let mut state = state.write().await;

    if state.connected() {
        if state.time_to_disconnect() {
            info!("disconnecting from region server");
            state.set_disconnected();
        }
    }
}

pub fn recv_heartbeat_request(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |(_addr, req)| {
        let state = state.clone();
        async move { async_recv_heartbeat_request_impl(state, req).await }
    });
}

async fn async_recv_heartbeat_request_impl(
    state: Arc<RwLock<State>>,
    _: HeartbeatRequest,
) -> Result<HeartbeatResponse, ResponseError> {
    info!("Heartbeat request received from region server, sending response");
    let mut state = state.write().await;
    state.heard_from_region_server();
    Ok(HeartbeatResponse)
}
