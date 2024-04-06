use log::warn;

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server, http_log_util};

use config::GATEWAY_SECRET;
use auth_server_http_proto::{UserNameForgotRequest, UserNameForgotResponse};

use crate::state::State;

pub fn user_name_forgot(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserNameForgotRequest,
) -> Result<UserNameForgotResponse, ResponseError> {
    if incoming_request.gateway_secret() != GATEWAY_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    http_log_util::recv_req("auth_server", "gateway", "user_name_forgot");

    let _state = state.read().await;

    http_log_util::send_res("auth_server", "gateway", "user_name_forgot");

    Ok(UserNameForgotResponse::new())
}
