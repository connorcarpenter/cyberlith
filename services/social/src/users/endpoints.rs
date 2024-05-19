use config::{REGION_SERVER_SECRET, SESSION_SERVER_GLOBAL_SECRET};
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;
use std::net::SocketAddr;

use social_server_http_proto::{
    UserConnectedRequest, UserConnectedResponse, UserDisconnectedRequest, UserDisconnectedResponse,
};

use crate::state::State;

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

    // TODO: store user connection details

    // responding
    return Ok(UserConnectedResponse);
}

pub fn recv_user_disconnected_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_user_disconnected_request_impl(state, addr, req).await }
    });
}

async fn async_recv_user_disconnected_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: UserDisconnectedRequest,
) -> Result<UserDisconnectedResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    // TODO: process

    // responding
    return Ok(UserDisconnectedResponse);
}
