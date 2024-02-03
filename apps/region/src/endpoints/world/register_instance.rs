use std::net::SocketAddr;

use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    WorldRegisterInstanceRequest,
    WorldRegisterInstanceResponse,
};
use config::WORLD_SERVER_SECRET;

use crate::{instances::WorldInstance, state::State};

pub fn world_register_instance(
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.endpoint(
        move |(addr, req)| {
            let state = state.clone();
            async move {
                async_impl(addr, state, req).await
            }
        }
    );
}

async fn async_impl(
    incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: WorldRegisterInstanceRequest
) -> Result<WorldRegisterInstanceResponse, ResponseError> {

    if incoming_request.world_secret() != WORLD_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();
    let public_url = incoming_request.public_url();

    info!(
        "register instance request received from world server: (incoming: {:?}, http: {:?}, public_url: {:?})",
        incoming_addr, http_addr, public_url
    );

    let server_instance = WorldInstance::new(http_addr, http_port, public_url);

    let mut state = state.write().await;
    state.register_world_instance(server_instance);

    info!("Sending register instance response to world server");

    Ok(WorldRegisterInstanceResponse)
}