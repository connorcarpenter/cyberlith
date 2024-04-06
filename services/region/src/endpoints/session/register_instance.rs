use std::net::SocketAddr;

use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::SESSION_SERVER_GLOBAL_SECRET;
use region_server_http_proto::{SessionRegisterInstanceRequest, SessionRegisterInstanceResponse};

use crate::{instances::SessionInstance, state::State};

pub fn session_register_instance(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(addr, req)| {
        let state = state.clone();
        async move { async_impl(addr, state, req).await }
    });
}

async fn async_impl(
    incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: SessionRegisterInstanceRequest,
) -> Result<SessionRegisterInstanceResponse, ResponseError> {
    if incoming_request.global_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let instance_secret = incoming_request.instance_secret();
    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();
    let public_webrtc_url = incoming_request.public_webrtc_url();

    info!(
        "register instance request received from session server: (incoming: {:?}, instance secret: {:?}, http: {:?}, public_webrtc_url: {:?})",
        incoming_addr, instance_secret, http_addr, public_webrtc_url
    );

    let server_instance =
        SessionInstance::new(instance_secret, http_addr, http_port, public_webrtc_url);

    let mut state = state.write().await;
    state.register_session_instance(server_instance);

    info!("Sending register instance response to session server");

    Ok(SessionRegisterInstanceResponse)
}
