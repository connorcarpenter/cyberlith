use logging::warn;

use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, smol::lock::RwLock, Server};

use auth_server_http_proto::{AccessTokenValidateRequest, AccessTokenValidateResponse};
use config::GATEWAY_SECRET;

use crate::{error::AuthServerError, state::State, types::AccessToken};

pub fn access_token_validate(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: AccessTokenValidateRequest,
) -> Result<AccessTokenValidateResponse, ResponseError> {
    if incoming_request.gateway_secret() != GATEWAY_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    http_log_util::recv_req("auth_server", "gateway", "access_token_validate");

    let mut state = state.write().await;
    let response = match state.access_token_validate(incoming_request) {
        Ok(_) => Ok(AccessTokenValidateResponse::new()),
        Err(AuthServerError::TokenSerdeError) => Err(ResponseError::SerdeError),
        Err(AuthServerError::TokenNotFound) => Err(ResponseError::Unauthenticated),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", "gateway", "access_token_validate");
    return response;
}

impl State {
    fn access_token_validate(
        &mut self,
        request: AccessTokenValidateRequest,
    ) -> Result<(), AuthServerError> {
        let Some(access_token) = AccessToken::from_str(&request.access_token) else {
            return Err(AuthServerError::TokenSerdeError);
        };

        if !self.has_access_token(&access_token) {
            return Err(AuthServerError::TokenNotFound);
        }

        return Ok(());
    }
}
