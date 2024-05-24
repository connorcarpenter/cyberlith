
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};

use social_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

use crate::state::State;

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
