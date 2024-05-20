use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbySendMessageRequest {
    session_instance_secret: String,
    user_id: UserId,
    message: String,
}

impl MatchLobbySendMessageRequest {
    pub fn new(session_instance_secret: &str, user_id: UserId, message: String) -> Self {
        Self {
            session_instance_secret: session_instance_secret.to_string(),
            user_id,
            message,
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
pub struct MatchLobbySendMessageResponse;

// Traits
impl ApiRequest for MatchLobbySendMessageRequest {
    type Response = MatchLobbySendMessageResponse;

    fn name() -> &'static str {
        "MatchLobbySendMessageRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_send_message"
    }
}

impl ApiResponse for MatchLobbySendMessageResponse {
    fn name() -> &'static str {
        "MatchLobbySendMessageResponse"
    }
}
