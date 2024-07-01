use auth_server_types::UserId;
use http_client::ResponseError;
use http_server::{
    async_dup::Arc, executor::smol::lock::RwLock, log_util, ApiRequest, ApiResponse, ApiServer,
    Server,
};

use auth_server_http_proto::{AccessTokenValidateRequest, AccessTokenValidateResponse};

use crate::{error::AuthServerError, state::State};

pub fn access_token_validate(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: AccessTokenValidateRequest,
) -> Result<AccessTokenValidateResponse, ResponseError> {
    log_util::recv_req(
        "auth_server",
        &AccessTokenValidateRequest::endpoint_key(),
        AccessTokenValidateRequest::name(),
    );

    let mut state = state.write().await;
    let response = match state.access_token_validate(incoming_request) {
        Ok(user_id) => Ok(AccessTokenValidateResponse::new(user_id)),
        Err(AuthServerError::TokenNotFound) => Err(ResponseError::Unauthenticated),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    log_util::send_res("auth_server", AccessTokenValidateResponse::name());
    return response;
}

impl State {
    fn access_token_validate(
        &mut self,
        request: AccessTokenValidateRequest,
    ) -> Result<UserId, AuthServerError> {
        let access_token = request.access_token;

        let Some(user_id) = self.get_access_token(&access_token) else {
            return Err(AuthServerError::TokenNotFound);
        };
        return Ok(user_id.clone());
    }
}
