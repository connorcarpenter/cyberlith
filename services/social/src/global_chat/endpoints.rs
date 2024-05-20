use std::net::SocketAddr;

use config::SESSION_SERVER_GLOBAL_SECRET;
use http_client::ResponseError;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiServer, Server};
use logging::warn;

use social_server_http_proto::{GlobalChatSendMessageRequest, GlobalChatSendMessageResponse};

use crate::state::State;

pub fn recv_global_chat_send_message_request(
    host_name: &str,
    server: &mut Server,
    state: Arc<RwLock<State>>,
) {
    server.api_endpoint(host_name, None, move |addr, req| {
        let state = state.clone();
        async move { async_recv_global_chat_send_message_request_impl(state, addr, req).await }
    });
}

async fn async_recv_global_chat_send_message_request_impl(
    state: Arc<RwLock<State>>,
    incoming_addr: SocketAddr,
    request: GlobalChatSendMessageRequest,
) -> Result<GlobalChatSendMessageResponse, ResponseError> {
    if request.session_secret() != SESSION_SERVER_GLOBAL_SECRET {
        warn!("invalid request secret");
        return Err(ResponseError::Unauthenticated);
    }

    let mut state = state.write().await;
    let Some(session_server_id) = state.session_servers.get_session_server_id(&incoming_addr) else {
        warn!("session server not found for incoming address: {:?}", incoming_addr);
        return Err(ResponseError::Unauthenticated);
    };

    state
        .global_chat
        .send_message(session_server_id, request.user_id(), request.message());

    // responding
    return Ok(GlobalChatSendMessageResponse);
}
