use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, smol::lock::RwLock, Server};

use config::{GATEWAY_SECRET, AUTH_SERVER_SECRET};
use auth_server_http_proto::{UserRegisterRequest, UserRegisterResponse};

use crate::state::State;

pub fn user_register(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserRegisterRequest,
) -> Result<UserRegisterResponse, ResponseError> {
    if incoming_request.gateway_secret() != GATEWAY_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    info!("user_register request <- gateway");

    let state = state.read().await;

    info!("user_register response -> gateway");

    Ok(UserRegisterResponse::new())
}
