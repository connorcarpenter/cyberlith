use config::SESSION_SERVER_GLOBAL_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;
use std::net::SocketAddr;

use social_server_http_proto::{
    MatchLobbyCreateRequest, MatchLobbyCreateResponse, MatchLobbyJoinRequest,
    MatchLobbyJoinResponse, MatchLobbyLeaveRequest, MatchLobbyLeaveResponse,
    MatchLobbySendMessageRequest, MatchLobbySendMessageResponse,
};

use crate::state::State;

pub fn recv_match_lobby_create_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_create_request_impl(state, addr, req).await }
    });
}

async fn async_recv_match_lobby_create_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: MatchLobbyCreateRequest,
) -> Result<MatchLobbyCreateResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    let new_match_lobby_id = state
        .match_lobbies
        .create(request.match_name(), request.creator_user_id());

    // responding
    return Ok(MatchLobbyCreateResponse::new(new_match_lobby_id));
}

pub fn recv_match_lobby_join_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_join_request_impl(state, addr, req).await }
    });
}

async fn async_recv_match_lobby_join_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: MatchLobbyJoinRequest,
) -> Result<MatchLobbyJoinResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    state
        .match_lobbies
        .join(request.match_lobby_id(), request.user_id());

    // responding
    return Ok(MatchLobbyJoinResponse);
}

pub fn recv_match_lobby_leave_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_leave_request_impl(state, addr, req).await }
    });
}

async fn async_recv_match_lobby_leave_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: MatchLobbyLeaveRequest,
) -> Result<MatchLobbyLeaveResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    state.match_lobbies.leave(request.user_id());

    // responding
    return Ok(MatchLobbyLeaveResponse);
}

pub fn recv_match_lobby_send_message_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_send_message_request_impl(state, addr, req).await }
    });
}

async fn async_recv_match_lobby_send_message_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: MatchLobbySendMessageRequest,
) -> Result<MatchLobbySendMessageResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;

    state
        .match_lobbies
        .send_message(request.user_id(), request.message());

    // responding
    return Ok(MatchLobbySendMessageResponse);
}
