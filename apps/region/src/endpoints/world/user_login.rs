
use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::{async_dup::Arc, Server, smol::lock::RwLock};

use region_server_http_proto::{
    WorldUserLoginRequest,
    WorldUserLoginResponse,
};
use world_server_http_proto::IncomingUserRequest;
use config::{REGION_SERVER_SECRET, SESSION_SERVER_SECRET};

use crate::state::State;

pub fn world_user_login(
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.endpoint(
        move |(_addr, req)| {
            let state = state.clone();
            async move {
                async_impl(state, req).await
            }
        }
    );
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: WorldUserLoginRequest
) -> Result<WorldUserLoginResponse, ResponseError> {

    if incoming_request.session_secret() != SESSION_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    info!("world user login request received from session server");

    let state = state.read().await;
    let Some(world_server) = state.get_available_world_server() else {
        warn!("no available world server");
        return Err(ResponseError::InternalServerError("no available world server".to_string()));
    };

    let world_server_http_addr = world_server.http_addr();
    let world_server_http_port = world_server.http_port();
    let world_server_public_url = world_server.public_url();

    info!("sending incoming user request to world server");

    let temp_token = crypto::generate_random_string(16);

    let request = IncomingUserRequest::new(REGION_SERVER_SECRET, &temp_token);

    let Ok(outgoing_response) = HttpClient::send(&world_server_http_addr, world_server_http_port, request).await else {
        warn!("failed incoming user request to world server");
        return Err(ResponseError::InternalServerError("failed incoming user request to world server".to_string()));
    };

    info!("Received incoming user response from world server");

    info!("Sending user login response to session server");

    // TODO: end of part we need to get rid of

    Ok(WorldUserLoginResponse::new(&world_server_public_url, &temp_token))
}
