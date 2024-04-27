use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, smol::lock::RwLock, ApiServer, Server, ApiResponse, ApiRequest};
use logging::{info, warn};

use auth_server_http_proto::{UserNameForgotRequest, UserNameForgotResponse};

use crate::{error::AuthServerError, state::State};

pub fn user_name_forgot(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserNameForgotRequest,
) -> Result<UserNameForgotResponse, ResponseError> {
    http_log_util::recv_req("auth_server", &UserNameForgotRequest::endpoint_key(), UserNameForgotRequest::name());

    let mut state = state.write().await;
    let response = match state.user_name_forgot(incoming_request) {
        Ok(()) => Ok(UserNameForgotResponse::new()),
        Err(AuthServerError::EmailNotFound) => {
            Ok(UserNameForgotResponse::new()) // we don't want to leak if an email is in the system or not
        }
        Err(AuthServerError::EmailSendFailed(inner_message)) => Err(
            ResponseError::InternalServerError(format!("Email send failed: {}", inner_message)),
        ),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", UserNameForgotResponse::name());
    return response;
}

impl State {
    fn user_name_forgot(&mut self, request: UserNameForgotRequest) -> Result<(), AuthServerError> {
        let user_email = request.email;

        if !self.email_to_id_map.contains_key(&user_email) {
            return Err(AuthServerError::EmailNotFound);
        }

        let username = self.get_user_name_by_email(&user_email);

        let email_subject = "Cyberlith Username Recovery"; // TODO: put into config
        let sending_email = "admin@cyberlith.com"; // TODO: put into config
        let link_url = "https://cyberlith.com"; // TODO: put into config

        info!(
            "sending forgotten username to user's email: {:?}",
            &user_email
        );

        let text_msg = self.email_catalog.user_name_forgot_txt(&username, link_url);
        let html_msg = self
            .email_catalog
            .user_name_forgot_html(&username, link_url);

        match email::send(
            sending_email,
            &user_email,
            email_subject,
            &text_msg,
            &html_msg,
        ) {
            Ok(_response) => {
                info!("email send success!");
                return Ok(());
            }
            Err(err) => {
                warn!("email send failed: {:?}", err);
                return Err(AuthServerError::EmailSendFailed(err.to_string()));
            }
        }
    }
}
