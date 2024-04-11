use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, smol::lock::RwLock, Server};

use auth_server_http_proto::{UserRegisterRequest, UserRegisterResponse};
use config::GATEWAY_SECRET;

use crate::{error::AuthServerError, state::State, types::TempRegistration};

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

    http_log_util::recv_req("auth_server", "gateway", "user_register");

    let mut state = state.write().await;
    let response = match state.user_register(incoming_request) {
        Ok(_) => Ok(UserRegisterResponse::new()),
        Err(AuthServerError::UsernameAlreadyExists) => Err(ResponseError::InternalServerError(
            "UsernameAlreadyExists".to_string(),
        )),
        Err(AuthServerError::EmailAlreadyExists) => Err(ResponseError::InternalServerError(
            "EmailAlreadyExists".to_string(),
        )),
        Err(AuthServerError::EmailSendFailed(inner_message)) => Err(
            ResponseError::InternalServerError(format!("Email send failed: {}", inner_message)),
        ),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", "gateway", "user_register");
    return response;
}

impl State {
    fn user_register(&mut self, request: UserRegisterRequest) -> Result<(), AuthServerError> {
        // TODO: validate data?
        // TODO: hash password?
        // TODO: expire registration token?

        if self.username_to_id_map.contains_key(&request.username) {
            return Err(AuthServerError::UsernameAlreadyExists);
        }
        if self.email_to_id_map.contains_key(&request.email) {
            return Err(AuthServerError::EmailAlreadyExists);
        }

        let reg_token = self.create_new_register_token();

        let temp_reg = TempRegistration::from(request);

        let email_subject = "Cyberlith Email Verification"; // TODO: put into config
        let sending_email = "admin@cyberlith.com"; // TODO: put into config
        let username = temp_reg.name.clone();
        let user_email: String = temp_reg.email.clone();
        let reg_token_str = reg_token.to_string();
        let link_url = format!("register_token={}", reg_token_str); // TODO: replace with working URL from config

        info!(
            "sending registration token to user's email: {:?}",
            &user_email
        );

        let text_msg = self
            .email_catalog
            .register_verification_txt(&username, &link_url);
        let html_msg = self
            .email_catalog
            .register_verification_html(&username, &link_url);

        match email::send(
            sending_email,
            &user_email,
            email_subject,
            &text_msg,
            &html_msg,
        ) {
            Ok(_response) => {
                info!("email send success!");

                self.store_register_token(reg_token, temp_reg);
                self.username_to_id_map.insert(username, None);
                self.email_to_id_map.insert(user_email, None);

                return Ok(());
            }
            Err(err) => {
                warn!("email send failed: {:?}", err);
                return Err(AuthServerError::EmailSendFailed(err.to_string()));
            }
        }
    }
}
