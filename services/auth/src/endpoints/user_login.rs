use auth_server_db::UserId;
use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, ApiServer, Server};
use logging::info;

use auth_server_http_proto::{UserLoginRequest, UserLoginResponse};

use crate::{error::AuthServerError, state::State, types::AccessToken, types::RefreshToken};

pub fn user_login(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(host_name, None, move |(_addr, req)| {
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
            let refresh_token = refresh_token.to_string();
            let access_token = access_token.to_string();
            Ok(UserLoginResponse::new(&refresh_token, &access_token))
        }
        Err(AuthServerError::UsernameOrEmailNotFound) | Err(AuthServerError::PasswordIncorrect) => Err(ResponseError::Unauthenticated),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    return response;
}

impl State {
    fn user_login(
        &mut self,
        request: UserLoginRequest,
    ) -> Result<(RefreshToken, AccessToken), AuthServerError> {
        let handle = request.handle;
        let password = request.password;

        // find user_id for given handle
        let user_id: UserId;
        if let Some(Some(id)) = self.username_to_id_map.get(&handle) {
            user_id = *id;
        } else if let Some(Some(id)) = self.email_to_id_map.get(&handle) {
            user_id = *id;
        } else {
            return Err(AuthServerError::UsernameOrEmailNotFound);
        }

        info!(
            "user_login: with handle {:?}, found user_id: {:?}",
            handle, user_id
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
