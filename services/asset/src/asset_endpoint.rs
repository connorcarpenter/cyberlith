use logging::info;

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server, ApiServer};

use asset_server_http_proto::{AssetRequest, AssetResponse};

use crate::state::State;

pub fn handle_asset_request(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_handle_asset_request_impl(state, req).await }
    });
}

async fn async_handle_asset_request_impl(
    state: Arc<RwLock<State>>,
    request: AssetRequest,
) -> Result<AssetResponse, ResponseError> {
    info!("Asset request received: {:?}, sending response", request);

    let mut state = state.write().await;
    state.handle_asset_request(request)
}
