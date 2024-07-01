use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;

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
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_create_request_impl(state, req).await }
    });
}

async fn async_recv_match_lobby_create_request_impl(
    state: Arc<RwLock<State>>,
    request: MatchLobbyCreateRequest,
) -> Result<MatchLobbyCreateResponse, ResponseError> {
    let mut state = state.write().await;

    let Some(session_server_id) = state
        .session_servers
        .get_session_server_id(&request.session_instance_secret())
    else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    let new_match_lobby_id = state.match_lobbies.create(
        session_server_id,
        request.match_name(),
        request.creator_user_id(),
    );

    // responding
    return Ok(MatchLobbyCreateResponse::new(new_match_lobby_id));
}

pub fn recv_match_lobby_join_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_join_request_impl(state, req).await }
    });
}

async fn async_recv_match_lobby_join_request_impl(
    state: Arc<RwLock<State>>,
    request: MatchLobbyJoinRequest,
) -> Result<MatchLobbyJoinResponse, ResponseError> {
    let mut state = state.write().await;

    let Some(session_server_id) = state
        .session_servers
        .get_session_server_id(&request.session_instance_secret())
    else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    state.match_lobbies.join(
        session_server_id,
        request.match_lobby_id(),
        request.user_id(),
    );

    // responding
    return Ok(MatchLobbyJoinResponse);
}

pub fn recv_match_lobby_leave_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_leave_request_impl(state, req).await }
    });
}

async fn async_recv_match_lobby_leave_request_impl(
    state: Arc<RwLock<State>>,
    request: MatchLobbyLeaveRequest,
) -> Result<MatchLobbyLeaveResponse, ResponseError> {
    let mut state = state.write().await;

    let Some(session_server_id) = state
        .session_servers
        .get_session_server_id(&request.session_instance_secret())
    else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    state
        .match_lobbies
        .leave(session_server_id, request.user_id());

    // responding
    return Ok(MatchLobbyLeaveResponse);
}

pub fn recv_match_lobby_send_message_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_match_lobby_send_message_request_impl(state, req).await }
    });
}

async fn async_recv_match_lobby_send_message_request_impl(
    state: Arc<RwLock<State>>,
    request: MatchLobbySendMessageRequest,
) -> Result<MatchLobbySendMessageResponse, ResponseError> {
    let mut state = state.write().await;

    let Some(session_server_id) = state
        .session_servers
        .get_session_server_id(&request.session_instance_secret())
    else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    state
        .match_lobbies
        .send_message(session_server_id, request.user_id(), request.message());

    // responding
    return Ok(MatchLobbySendMessageResponse);
}
