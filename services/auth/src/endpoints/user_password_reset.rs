use logging::warn;

use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, smol::lock::RwLock, ApiServer, Server};

use auth_server_http_proto::{UserPasswordResetRequest, UserPasswordResetResponse};
use config::GATEWAY_SECRET;

use crate::{error::AuthServerError, state::State, types::ResetPasswordToken};

pub fn user_password_reset(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserPasswordResetRequest,
) -> Result<UserPasswordResetResponse, ResponseError> {
    if incoming_request.gateway_secret() != GATEWAY_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    http_log_util::recv_req("auth_server", "gateway", "user_password_reset");

    let mut state = state.write().await;
    let response = match state.user_password_reset(incoming_request) {
        Ok(()) => Ok(UserPasswordResetResponse::new()),
        Err(AuthServerError::TokenNotFound) => {
            Err(ResponseError::InternalServerError("NotFound".to_string()))
        }
        Err(AuthServerError::TokenSerdeError) => {
            Err(ResponseError::InternalServerError("SerdeError".to_string()))
        }
        Err(AuthServerError::EmailSendFailed(inner_message)) => Err(
            ResponseError::InternalServerError(format!("Email send failed: {}", inner_message)),
        ),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", "gateway", "user_password_reset");
    return response;
}

impl State {
    fn user_password_reset(
        &mut self,
        request: UserPasswordResetRequest,
    ) -> Result<(), AuthServerError> {
        let new_password = request.new_password;
        let Some(reset_token) = ResetPasswordToken::from_str(&request.reset_password_token) else {
            return Err(AuthServerError::TokenSerdeError);
        };
        let Some(user_id) = self.remove_reset_password_token(&reset_token) else {
            return Err(AuthServerError::TokenNotFound);
        };

        // set new password
        self.set_user_password(user_id, new_password);

        return Ok(());
    }
}
