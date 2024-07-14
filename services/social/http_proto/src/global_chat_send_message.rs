use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;
use social_server_types::{MessageId, Timestamp};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct GlobalChatSendMessageRequest {
    session_instance_secret: String,
    user_id: UserId,
    message: String,
}

impl GlobalChatSendMessageRequest {
    pub fn new(session_instance_secret: &str, user_id: UserId, message: &str) -> Self {
        Self {
            session_instance_secret: session_instance_secret.to_string(),
            user_id,
            message: message.to_string(),
        }
    }

    pub fn session_instance_secret(&self) -> &str {
        &self.session_instance_secret
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
pub struct GlobalChatSendMessageResponse {
    pub global_chat_message_id: MessageId,
    pub timestamp: Timestamp,
}

impl GlobalChatSendMessageResponse {
    pub fn new(global_chat_message_id: MessageId, timestamp: Timestamp) -> Self {
        Self {
            global_chat_message_id,
            timestamp,
        }
    }
}

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
