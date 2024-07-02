
use config::REGION_SERVER_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::{info, warn};

use social_server_http_proto::{
    UserConnectedRequest, UserConnectedResponse, UserDisconnectedRequest, UserDisconnectedResponse,
};

use crate::state::State;

// User Connection

pub fn recv_user_connected_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_user_connected_request_impl(state, req).await }
    });
}

async fn async_recv_user_connected_request_impl(
    state: Arc<RwLock<State>>,
    request: UserConnectedRequest,
) -> Result<UserConnectedResponse, ResponseError> {
    if request.region_secret() != REGION_SERVER_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    // setting last heard
    state.region_server.heard_from_region_server();

    {
        let user_id_u64: u64 = request.user_id().into();
        let user_name = request.user_name();
        info!("User connected: {} - {:?}", user_id_u64, user_name);
    }

    state.users.connect_user(&request.user_id(), request.user_name());

    // responding
    return Ok(UserConnectedResponse);
}

// User Disconnection

pub fn recv_user_disconnected_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_user_disconnected_request_impl(state, req).await }
    });
}

async fn async_recv_user_disconnected_request_impl(
    state: Arc<RwLock<State>>,
    request: UserDisconnectedRequest,
) -> Result<UserDisconnectedResponse, ResponseError> {
    let mut state = state.write().await;

    let Some(session_server_id) = state
        .session_servers
        .get_session_server_id(&request.session_instance_secret())
    else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    state
        .users
        .disconnect_user(session_server_id, request.user_id());

    // responding
    return Ok(UserDisconnectedResponse);
}
