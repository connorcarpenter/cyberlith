use std::net::SocketAddr;

use config::SESSION_SERVER_GLOBAL_SECRET;
use http_client::ResponseError;
use http_server::{ApiServer, Server, async_dup::Arc, executor::smol::lock::RwLock};
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

    state.global_chat.send_message(request.user_id(), request.message());

    // responding
    return Ok(GlobalChatSendMessageResponse);
}