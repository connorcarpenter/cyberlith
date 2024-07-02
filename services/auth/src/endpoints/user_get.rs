
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::info;

use auth_server_types::UserRole;
use auth_server_http_proto::{UserGetRequest, UserGetResponse};

use crate::{error::AuthServerError, state::State};

pub fn user_get(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: UserGetRequest,
) -> Result<UserGetResponse, ResponseError> {
    let mut state = state.write().await;
    let response = match state.user_get(incoming_request) {
        Ok((name, email, role)) => {
            Ok(UserGetResponse::new(name, email, role))
        }
        Err(_) => {
            panic!("unhandled error for this endpoint");
        }
    };

    return response;
}

impl State {

    fn user_get(
        &mut self,
        request: UserGetRequest,
    ) -> Result<(String, String, UserRole), AuthServerError> {
        let user_id = request.user_id;

        info!("user_get: with user_id: {:?}", user_id);

        let Some(user) = self.database_manager.get_user(&user_id) else {
            return Err(AuthServerError::UserIdNotFound);
        };

        let name = user.username().to_string();
        let email = user.email().to_string();
        let role = user.role().clone().into();

        Ok((name, email, role))
    }
}
