
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
    server.api_endpoint(host_name, None, move |_addr, req| {
        let state = state.clone();
        async move { async_recv_global_chat_send_message_request_impl(state, req).await }
    });
}

async fn async_recv_global_chat_send_message_request_impl(
    state: Arc<RwLock<State>>,
    request: GlobalChatSendMessageRequest,
) -> Result<GlobalChatSendMessageResponse, ResponseError> {

    let mut state = state.write().await;

    let Some(session_server_id) = state.session_servers.get_session_server_id(&request.session_instance_secret()) else {
        warn!("invalid request instance secret");
        return Err(ResponseError::Unauthenticated);
    };

    let (msg_id, timestamp) = state
        .global_chat
        .send_message(session_server_id, request.user_id(), request.message());

    // responding
    let response = GlobalChatSendMessageResponse::new(
        msg_id,
        timestamp,
    );
    return Ok(response);
}
