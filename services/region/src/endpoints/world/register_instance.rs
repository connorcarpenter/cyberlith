use std::net::SocketAddr;

use logging::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::WORLD_SERVER_GLOBAL_SECRET;
use region_server_http_proto::{WorldRegisterInstanceRequest, WorldRegisterInstanceResponse};

use crate::{instances::WorldInstance, state::State};

pub fn world_register_instance(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(addr, req)| {
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
    let public_webrtc_url = incoming_request.public_webrtc_url();

    info!(
        "register instance request received from world server: (incoming: {:?}, http: {:?}, public_webrtc_url: {:?})",
        incoming_addr, http_addr, public_webrtc_url
    );

    let server_instance =
        WorldInstance::new(instance_secret, http_addr, http_port, public_webrtc_url);

    let mut state = state.write().await;
    state.register_world_instance(server_instance);

    info!("Sending register instance response to world server");

    Ok(WorldRegisterInstanceResponse)
}
