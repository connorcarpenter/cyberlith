use auth_server_types::UserId;
use http_client::ResponseError;
use http_server::{
    async_dup::Arc, executor::smol::lock::RwLock, http_log_util, ApiRequest, ApiResponse,
    ApiServer, Server,
};

use auth_server_http_proto::{AccessToken, RefreshTokenGrantRequest, RefreshTokenGrantResponse};

use crate::{error::AuthServerError, state::State};

pub fn refresh_token_grant(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: RefreshTokenGrantRequest,
) -> Result<RefreshTokenGrantResponse, ResponseError> {
    http_log_util::recv_req(
        "auth_server",
        &RefreshTokenGrantRequest::endpoint_key(),
        RefreshTokenGrantRequest::name(),
    );

    let mut state = state.write().await;
    let response = match state.refresh_token_grant(incoming_request) {
        Ok((user_id, access_token)) => Ok(RefreshTokenGrantResponse::new(user_id, access_token)),
        Err(AuthServerError::TokenNotFound) => Err(ResponseError::Unauthenticated),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", RefreshTokenGrantResponse::name());
    return response;
}

impl State {
    fn refresh_token_grant(
        &mut self,
        request: RefreshTokenGrantRequest,
    ) -> Result<(UserId, AccessToken), AuthServerError> {
        let refresh_token = request.refresh_token;

        if !self.has_refresh_token(&refresh_token) {
            return Err(AuthServerError::TokenNotFound);
        }

        let user_id = self.get_user_id_by_refresh_token(&refresh_token).unwrap();
        let user_id = *user_id;

        let access_token = self.create_and_store_access_token(&user_id);

        return Ok((user_id, access_token));
    }
}
