use logging::{info, warn};

use config::TargetEnv;
use http_client::ResponseError;
use http_server::{
    async_dup::Arc, executor::smol::lock::RwLock, http_log_util, ApiRequest, ApiResponse,
    ApiServer, Server,
};

use auth_server_http_proto::{UserRegisterRequest, UserRegisterResponse};
use validation::{EmailValidation, PasswordValidation, UsernameValidation, Validator};

use crate::{error::AuthServerError, state::State, types::TempRegistration};

pub fn user_register(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserRegisterRequest,
) -> Result<UserRegisterResponse, ResponseError> {
    http_log_util::recv_req(
        "auth_server",
        &UserRegisterRequest::endpoint_key(),
        UserRegisterRequest::name(),
    );

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

    http_log_util::send_res("auth_server", UserRegisterResponse::name());
    return response;
}

impl State {
    fn user_register(&mut self, request: UserRegisterRequest) -> Result<(), AuthServerError> {
        let username = request.username.to_ascii_lowercase();
        let email = request.email;
        let password = request.password;

        if !UsernameValidation::allows_text(&username) {
            return Err(AuthServerError::UsernameInvalidCharacters);
        }
        if !EmailValidation::allows_text(&email) {
            return Err(AuthServerError::EmailInvalidCharacters);
        }
        if !PasswordValidation::allows_text(&password) {
            return Err(AuthServerError::PasswordInvalidCharacters);
        }
        // TODO: validate data more?

        if self.username_to_id_map.contains_key(&password) {
            return Err(AuthServerError::UsernameAlreadyExists);
        }
        if self.email_to_id_map.contains_key(&email) {
            return Err(AuthServerError::EmailAlreadyExists);
        }

        let reg_token = self.create_new_register_token();

        let temp_reg = TempRegistration::new(&username, &email, &password)?;

        let email_subject = "Cyberlith Email Verification"; // TODO: put into config
        let sending_email = "admin@cyberlith.com"; // TODO: put into config
        let username = temp_reg.name.clone();
        let user_email: String = temp_reg.email.clone();
        let reg_token_str = reg_token.to_string();

        // on local, should be http://127.0.0.1:14196/?register_token={}
        // on prod, should be https://www.cyberlith.com/?register_token={}
        let link_url = format!(
            "{}/?register_token={}",
            TargetEnv::gateway_url(),
            reg_token_str
        ); // TODO: replace with working URL from config

        info!("sending register token to user's email: {:?}", &user_email);

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
