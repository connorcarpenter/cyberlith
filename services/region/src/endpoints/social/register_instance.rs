use std::net::SocketAddr;

use config::SOCIAL_SERVER_GLOBAL_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;

use region_server_http_proto::{SocialRegisterInstanceRequest, SocialRegisterInstanceResponse};

use crate::state::State;

pub fn social_register_instance(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_impl(addr, state, req).await }
    });
}

async fn async_impl(
    _incoming_addr: SocketAddr,
    state: Arc<RwLock<State>>,
    incoming_request: SocialRegisterInstanceRequest,
) -> Result<SocialRegisterInstanceResponse, ResponseError> {
    if incoming_request.global_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let http_addr = incoming_request.http_addr();
    let http_port = incoming_request.http_port();

    // info!(
    //     "register instance request received from social server: (incoming: {:?}, http: {:?})",
    //     incoming_addr, http_addr
    // );

    let mut state = state.write().await;
    state.register_social_instance(http_addr, http_port).await;

    // info!("Sending register instance response to social server");

    Ok(SocialRegisterInstanceResponse)
}
