use std::net::SocketAddr;

use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::ASSET_SERVER_GLOBAL_SECRET;
use region_server_http_proto::{AssetRegisterInstanceRequest, AssetRegisterInstanceResponse};

use crate::{instances::AssetInstance, state::State};

pub fn asset_register_instance(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(addr, req)| {
        let state = state.clone();
        async move { async_impl(addr, state, req).await }
    });
}

async fn async_impl(
    incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: AssetRegisterInstanceRequest,
) -> Result<AssetRegisterInstanceResponse, ResponseError> {
    if incoming_request.global_secret() != ASSET_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();

    info!(
        "register instance request received from asset server: (incoming: {:?}, http: {:?})",
        incoming_addr, http_addr
    );

    let asset_instance = AssetInstance::new(http_addr, http_port);

    let mut state = state.write().await;
    state.register_asset_instance(asset_instance);

    info!("Sending register instance response to asset server");

    Ok(AssetRegisterInstanceResponse)
}
