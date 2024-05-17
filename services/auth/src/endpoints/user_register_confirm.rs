use logging::info;

use http_client::ResponseError;
use http_server::{async_dup::Arc, http_log_util, executor::smol::lock::RwLock, ApiServer, Server, ApiResponse, ApiRequest};
use auth_server_db::{AuthServerDbError, User, UserRole};
use auth_server_types::UserId;

use auth_server_http_proto::{AccessToken, RefreshToken, UserRegisterConfirmRequest, UserRegisterConfirmResponse};

use crate::{error::AuthServerError, state::State};

pub fn user_register_confirm(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserRegisterConfirmRequest,
) -> Result<UserRegisterConfirmResponse, ResponseError> {
    http_log_util::recv_req("auth_server", &UserRegisterConfirmRequest::endpoint_key(), UserRegisterConfirmRequest::name());

    let mut state = state.write().await;
    let response = match state.user_register_confirm(incoming_request).await {
        Ok((refresh_token, access_token)) => Ok(UserRegisterConfirmResponse::new(access_token, refresh_token)),
        Err(AuthServerError::TokenNotFound) => Err(ResponseError::InternalServerError(
            "TokenNotFound".to_string(),
        )),
        Err(AuthServerError::InsertedDuplicateUserId) => Err(ResponseError::InternalServerError(
            "InsertedDuplicateUserId".to_string(),
        )),
        Err(AuthServerError::Unknown(msg)) => Err(ResponseError::InternalServerError(format!(
            "Unknown Error: {:?}",
            msg
        ))),
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", UserRegisterConfirmResponse::name());

    response
}

impl State {
    pub async fn user_register_confirm(
        &mut self,
        request: UserRegisterConfirmRequest,
    ) -> Result<(RefreshToken, AccessToken), AuthServerError> {
        let reg_token = request.register_token;
        let Some(temp_reg) = self.remove_register_token(&reg_token) else {
            return Err(AuthServerError::TokenNotFound);
        };

        let new_user = User::new(
            &temp_reg.name,
            &temp_reg.email,
            &temp_reg.password,
            UserRole::Free,
        );
        let new_user_id: u64 = self
            .database_manager
            .create_user(new_user)
            .map_err(|err| match err {
                AuthServerDbError::InsertedDuplicateUserId => {
                    AuthServerError::InsertedDuplicateUserId
                }
            })?.into();
        let new_user_id = UserId::new(new_user_id);

        // add to username -> id map
        let Some(id_opt) = self.username_to_id_map.get_mut(&temp_reg.name) else {
            return Err(AuthServerError::Unknown(
                "username not found AFTER register confirm".to_string(),
            ));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown(
                "username already exists AFTER register confirm".to_string(),
            ));
        }
        *id_opt = Some(new_user_id);

        // add to email -> id map
        let Some(id_opt) = self.email_to_id_map.get_mut(&temp_reg.email) else {
            return Err(AuthServerError::Unknown(
                "email not found AFTER register confirm".to_string(),
            ));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown(
                "email already exists AFTER register confirm".to_string(),
            ));
        }
        *id_opt = Some(new_user_id);

        // add user data
        self.init_user_data(&new_user_id);

        // generate new access and refresh token for user
        let (refresh_token, access_token) = self.user_new_login_gen_tokens(&new_user_id);

        info!("new user created: {:?} - {:?}", new_user_id, temp_reg.name);

        Ok((refresh_token, access_token))
    }
}
