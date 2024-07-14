use config::REGION_SERVER_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;

use social_server_http_proto::{
    UserConnectedRequest, UserConnectedResponse, UserDisconnectedRequest, UserDisconnectedResponse,
    UserIsOnlineRequest, UserIsOnlineResponse,
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

    if state.users.is_user_online(&request.user_id()) {
        return Ok(UserConnectedResponse::already_connected());
    }

    state.users.connect_user(&request.user_id());

    // responding
    return Ok(UserConnectedResponse::success());
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

// User Is Online

pub fn recv_user_is_online_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_user_is_online_request_impl(state, req).await }
    });
}

async fn async_recv_user_is_online_request_impl(
    state: Arc<RwLock<State>>,
    request: UserIsOnlineRequest,
) -> Result<UserIsOnlineResponse, ResponseError> {
    let mut state = state.write().await;

    // setting last heard
    state.region_server.heard_from_region_server();

    if state.users.is_user_online(&request.user_id()) {
        return Ok(UserIsOnlineResponse::online());
    } else {
        return Ok(UserIsOnlineResponse::offline());
    }
}
