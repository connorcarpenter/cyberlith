use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct GlobalChatSendMessageRequest {
    session_secret: String,
    user_id: UserId,
    message: String,
}

impl GlobalChatSendMessageRequest {
    pub fn new(session_secret: &str, user_id: UserId, message: &str) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            user_id,
            message: message.to_string(),
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct GlobalChatSendMessageResponse;

// Traits
impl ApiRequest for GlobalChatSendMessageRequest {
    type Response = GlobalChatSendMessageResponse;

    fn name() -> &'static str {
        "GlobalChatSendMessageRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "global_chat_send_message"
    }
}

impl ApiResponse for GlobalChatSendMessageResponse {
    fn name() -> &'static str {
        "GlobalChatSendMessageResponse"
    }
}
