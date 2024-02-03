use std::net::SocketAddr;

use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    SessionRegisterInstanceRequest,
    SessionRegisterInstanceResponse,
};
use config::SESSION_SERVER_SECRET;

use crate::{instances::SessionInstance, state::State};

pub fn session_register_instance(
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
    incoming_request: SessionRegisterInstanceRequest
) -> Result<SessionRegisterInstanceResponse, ResponseError> {

    if incoming_request.session_secret() != SESSION_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();
    let signal_addr = incoming_request.signal_addr();
    let signal_port = incoming_request.signal_port();

    info!(
        "register instance request received from session server: (incoming: {:?}, http: {:?}, signal: {:?})",
        incoming_addr, http_addr, signal_addr
    );

    let server_instance = SessionInstance::new(http_addr, http_port, signal_addr, signal_port);

    let mut state = state.write().await;
    state.register_session_instance(server_instance);

    info!("Sending register instance response to session server");

    Ok(SessionRegisterInstanceResponse)
}