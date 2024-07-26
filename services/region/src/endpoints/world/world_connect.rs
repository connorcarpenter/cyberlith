use logging::{info, warn};

use config::{REGION_SERVER_SECRET, SOCIAL_SERVER_GLOBAL_SECRET};
use http_client::{HttpClient, ResponseError};
use http_server::{
    async_dup::Arc, executor::smol::lock::RwLock, ApiRequest, ApiResponse, ApiServer, Server,
};
use region_server_http_proto::{
    WorldConnectRequest as RegionWorldConnectRequest,
    WorldConnectResponse as RegionWorldConnectResponse,
};
use world_server_http_proto::{
    WorldConnectRequest as WorldWorldConnectRequest,
    WorldConnectResponse as WorldWorldConnectResponse,
};

use crate::state::State;

pub fn world_connect(host_name: &str, server: &mut Server, state: Arc<RwLock<State>>) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_impl(state, req).await }
    });
}

async fn async_impl(
    state: Arc<RwLock<State>>,
    incoming_request: RegionWorldConnectRequest,
) -> Result<RegionWorldConnectResponse, ResponseError> {
    if incoming_request.social_server_global_secret != SOCIAL_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let state = state.read().await;
    let Some(world_server) = state.get_available_world_server() else {
        warn!("no available world server");
        return Err(ResponseError::InternalServerError(
            "no available world server".to_string(),
        ));
    };

    let world_server_instance_secret = world_server.instance_secret();
    let world_server_http_addr = world_server.http_addr();
    let world_server_http_port = world_server.http_port();

    let lobby_id = incoming_request.lobby_id;

    let mut login_tokens_to_world_server = Vec::new();
    let mut login_tokens_in_response = Vec::new();
    for (session_instance_secret, user_ids) in incoming_request.user_ids {
        let Some(session_server) =
            state.get_session_server_from_instance_secret(&session_instance_secret)
        else {
            warn!("session server not found: {}", session_instance_secret);
            continue;
        };
        let mut user_login_tokens = Vec::new();
        for user_id in user_ids {
            let login_token = random::generate_random_string(16);
            user_login_tokens.push((user_id, login_token.clone()));
            login_tokens_in_response.push((user_id, login_token));
        }

        let session_server_addr = session_server.http_addr();
        let session_server_port = session_server.http_port();

        login_tokens_to_world_server.push((
            session_server_addr.to_string(),
            session_server_port,
            user_login_tokens,
        ));
    }

    let host = "region";
    let remote = "world";

    let world_server_request =
        WorldWorldConnectRequest::new(REGION_SERVER_SECRET, lobby_id, login_tokens_to_world_server);

    http_server::log_util::send_req(&host, &remote, WorldWorldConnectRequest::name());
    let Ok(_world_server_response) = HttpClient::send(
        world_server_http_addr,
        world_server_http_port,
        world_server_request,
    )
    .await
    else {
        return Err(ResponseError::InternalServerError(
            "failed world connect request to world server".to_string(),
        ));
    };
    http_server::log_util::recv_res(&host, &remote, WorldWorldConnectResponse::name());

    Ok(RegionWorldConnectResponse::new(
        world_server_instance_secret,
        login_tokens_in_response,
    ))
}
