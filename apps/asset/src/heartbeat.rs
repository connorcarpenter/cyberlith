use log::info;

use http_client::ResponseError;
use http_server::Server;

use crate::state::State;
use asset_server_http_proto::{HeartbeatRequest, HeartbeatResponse};
use http_server::async_dup::Arc;
use http_server::smol::lock::RwLock;

pub fn endpoint(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    _: HeartbeatRequest,
) -> Result<HeartbeatResponse, ResponseError> {
    info!("Heartbeat request received from region server, sending response");
    let mut state = state.write().await;
    state.heard_from_region_server();
    Ok(HeartbeatResponse)
}
