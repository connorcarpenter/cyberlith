use std::net::SocketAddr;

use log::info;

use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    WorldRegisterInstanceRequest,
    WorldRegisterInstanceResponse,
};

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
) -> Result<WorldRegisterInstanceResponse, ()> {

    let http_addr = incoming_request.http_addr();
    let signal_addr = incoming_request.signal_addr();

    info!(
        "register instance request received from world server: (incoming: {:?}, http: {:?}, signal: {:?})",
        incoming_addr, http_addr, signal_addr
    );

    let server_instance = WorldInstance::new(http_addr, signal_addr);

    let mut state = state.write().await;
    state.register_world_instance(server_instance);

    info!("Sending register instance response to world server");

    Ok(WorldRegisterInstanceResponse)
}