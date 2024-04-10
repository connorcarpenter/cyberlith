use log::{info, warn};

use http_client::ResponseError;
use http_server::{async_dup::Arc, smol::lock::RwLock, Server, http_log_util};
use crypto::U32Token;

use config::GATEWAY_SECRET;
use auth_server_http_proto::{UserRegisterConfirmRequest, UserRegisterConfirmResponse};
use auth_server_db::{AuthServerDbError, User, UserRole};

use crate::{state::State, error::AuthServerError, types::RegisterToken};

pub fn user_register_confirm(server: &mut Server, state: Arc<RwLock<State>>) {
    server.endpoint(move |(_addr, req)| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserRegisterConfirmRequest,
) -> Result<UserRegisterConfirmResponse, ResponseError> {
    if incoming_request.gateway_secret() != GATEWAY_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    http_log_util::recv_req("auth_server", "gateway", "user_register_confirm");

    let mut state = state.write().await;
    let response = match state.user_register_confirm(incoming_request) {
        Ok(()) => {
            Ok(UserRegisterConfirmResponse::new("faketoken"))
        }
        Err(AuthServerError::RegisterTokenSerdeError) => {
            Err(ResponseError::InternalServerError("TokenSerdeError".to_string()))
        }
        Err(AuthServerError::RegisterTokenNotFound) => {
            Err(ResponseError::InternalServerError("TokenNotFound".to_string()))
        }
        Err(AuthServerError::InsertedDuplicateUserId) => {
            Err(ResponseError::InternalServerError("InsertedDuplicateUserId".to_string()))
        }
        Err(AuthServerError::Unknown(msg)) => {
            Err(ResponseError::InternalServerError(format!("Unknown Error: {:?}", msg)))
        }
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    http_log_util::send_res("auth_server", "gateway", "user_register_confirm");

    response
}

impl State {
    pub fn user_register_confirm(&mut self, request: UserRegisterConfirmRequest) -> Result<(), AuthServerError> {

        let Some(reg_token) = U32Token::from_str(&request.register_token) else {
            return Err(AuthServerError::RegisterTokenSerdeError);
        };
        let reg_token = RegisterToken::from(reg_token);
        let Some(temp_reg) = self.remove_register_token(&reg_token) else {
            return Err(AuthServerError::RegisterTokenNotFound);
        };

        let new_user = User::new(&temp_reg.name, &temp_reg.email, &temp_reg.password, UserRole::Free);
        let new_user_id = self.database_manager.create_user(new_user).map_err(|err| {
            match err {
                AuthServerDbError::InsertedDuplicateUserId => AuthServerError::InsertedDuplicateUserId,
            }
        })?;

        // add to username -> id map
        let Some(id_opt) = self.username_to_id_map.get_mut(&temp_reg.name) else {
            return Err(AuthServerError::Unknown("username not found AFTER register confirm".to_string()));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown("username already exists AFTER register confirm".to_string()));
        }
        *id_opt = Some(new_user_id);

        // add to email -> id map
        let Some(id_opt) = self.email_to_id_map.get_mut(&temp_reg.email) else {
            return Err(AuthServerError::Unknown("email not found AFTER register confirm".to_string()));
        };
        if id_opt.is_some() {
            return Err(AuthServerError::Unknown("email already exists AFTER register confirm".to_string()));
        }
        *id_opt = Some(new_user_id);

        let new_user_id: u64 = new_user_id.into();
        info!("new user created: {:?} - {:?}", new_user_id, temp_reg.name);

        Ok(())
    }
}