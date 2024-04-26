use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, smol::lock::RwLock, ApiServer, Server};
use logging::{info, warn};

use auth_server_http_proto::{UserPasswordForgotRequest, UserPasswordForgotResponse};

use crate::{error::AuthServerError, state::State};

pub fn user_password_forgot(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(host_name, None, move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserPasswordForgotRequest,
) -> Result<UserPasswordForgotResponse, ResponseError> {
    http_log_util::recv_req("auth_server", "user_password_forgot");

    let mut state = state.write().await;
    let response = match state.user_password_forgot(incoming_request) {
        Ok(()) => Ok(UserPasswordForgotResponse::new()),
        Err(AuthServerError::EmailNotFound) => {
            Ok(UserPasswordForgotResponse::new()) // we don't want to leak if an email is in the system or not
        }
        Err(AuthServerError::EmailSendFailed(inner_message)) => Err(
            ResponseError::InternalServerError(format!("Email send failed: {}", inner_message)),
        ),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", "user_password_forgot");
    return response;
}

impl State {
    fn user_password_forgot(
        &mut self,
        request: UserPasswordForgotRequest,
    ) -> Result<(), AuthServerError> {
        let user_email = request.email;

        if !self.email_to_id_map.contains_key(&user_email) {
            return Err(AuthServerError::EmailNotFound);
        }

        let user_name = self.get_user_name_by_email(&user_email);
        let user_id = self.get_user_id_by_email(&user_email);
        let reset_token = self.create_new_reset_password_token();

        let email_subject = "Cyberlith Password Reset"; // TODO: put into config
        let sending_email = "admin@cyberlith.com"; // TODO: put into config
        let reset_token_str = reset_token.to_string();
        let link_url = format!("reset_password_token={}", reset_token_str); // TODO: replace with working URL from config

        info!(
            "sending reset password token to user's email: {:?}",
            &user_email
        );

        let text_msg = self
            .email_catalog
            .user_password_forgot_txt(&user_name, &link_url);
        let html_msg = self
            .email_catalog
            .user_password_forgot_html(&user_name, &link_url);

        match email::send(
            sending_email,
            &user_email,
            email_subject,
            &text_msg,
            &html_msg,
        ) {
            Ok(_response) => {
                info!("email send success!");

                self.store_reset_password_token(user_id, reset_token);

                return Ok(());
            }
            Err(err) => {
                warn!("email send failed: {:?}", err);
                return Err(AuthServerError::EmailSendFailed(err.to_string()));
            }
        }
    }
}
