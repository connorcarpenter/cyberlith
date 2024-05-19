use auth_server_types::UserId;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::info;
use validation::{EmailValidation, PasswordValidation, UsernameValidation, Validator};

use auth_server_http_proto::{AccessToken, RefreshToken, UserLoginRequest, UserLoginResponse};

use crate::{error::AuthServerError, state::State};

pub fn user_login(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserLoginRequest,
) -> Result<UserLoginResponse, ResponseError> {
    let mut state = state.write().await;
    let response = match state.user_login(incoming_request) {
        Ok((refresh_token, access_token)) => {
            Ok(UserLoginResponse::new(refresh_token, access_token))
        }
        Err(AuthServerError::UsernameOrEmailNotFound) | Err(AuthServerError::PasswordIncorrect) => {
            Err(ResponseError::Unauthenticated)
        }
        Err(AuthServerError::PasswordInvalidCharacters)
        | Err(AuthServerError::UsernameInvalidCharacters)
        | Err(AuthServerError::EmailInvalidCharacters) => Err(ResponseError::BadRequest),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    return response;
}

impl State {
    fn user_id_from_username_from_handle(&self, username: &str) -> Option<UserId> {
        let username = username.to_ascii_lowercase();

        if !UsernameValidation::allows_text(&username) {
            return None;
        }

        let user_id_opt = self.username_to_id_map.get(&username)?;
        *user_id_opt
    }

    fn user_id_from_email_from_handle(&self, email: &str) -> Option<UserId> {
        if !EmailValidation::allows_text(&email) {
            return None;
        }

        let user_id_opt = self.email_to_id_map.get(email)?;
        *user_id_opt
    }

    fn user_login(
        &mut self,
        request: UserLoginRequest,
    ) -> Result<(RefreshToken, AccessToken), AuthServerError> {
        let handle = request.handle;
        let password = request.password;

        // validate password
        if !PasswordValidation::allows_text(&password) {
            return Err(AuthServerError::PasswordInvalidCharacters);
        }

        // find user_id for given handle
        let user_id: UserId;
        if let Some(id) = self.user_id_from_username_from_handle(&handle) {
            user_id = id;
        } else if let Some(id) = self.user_id_from_email_from_handle(&handle) {
            user_id = id;
        } else {
            return Err(AuthServerError::UsernameOrEmailNotFound);
        }

        info!(
            "user_login: with handle {:?}, found user_id: {:?} .. password: {:?}",
            handle, user_id, password,
        );

        // check password
        let user = self.database_manager.get_user(&user_id).unwrap();
        if !user.check_password(&password) {
            return Err(AuthServerError::PasswordIncorrect);
        }

        // create and store new access token
        let (refresh_token, access_token) = self.user_new_login_gen_tokens(&user_id);
        Ok((refresh_token, access_token))
    }
}
