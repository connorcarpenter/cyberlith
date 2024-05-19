use std::net::SocketAddr;

use logging::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};

use config::WORLD_SERVER_GLOBAL_SECRET;
use region_server_http_proto::{WorldRegisterInstanceRequest, WorldRegisterInstanceResponse};

use crate::{instances::WorldInstance, state::State};

pub fn world_register_instance(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_impl(addr, state, req).await }
    });
}

async fn async_impl(
    incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: WorldRegisterInstanceRequest,
) -> Result<WorldRegisterInstanceResponse, ResponseError> {
    if incoming_request.global_secret() != WORLD_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let instance_secret = incoming_request.instance_secret();
    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();

    info!(
        "register instance request received from world server: (incoming: {:?}, http: {:?})",
        incoming_addr, http_addr,
    );

    let server_instance = WorldInstance::new(instance_secret, http_addr, http_port);

    let mut state = state.write().await;
    state.register_world_instance(server_instance);

    info!("Sending register instance response to world server");

    Ok(WorldRegisterInstanceResponse)
}
