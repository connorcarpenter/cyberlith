use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbySendMessageRequest {
    session_secret: String,
    match_lobby_id: MatchLobbyId,
    user_id: UserId,
    message: String,
}

impl MatchLobbySendMessageRequest {
    pub fn new(
        session_secret: &str,
        match_lobby_id: MatchLobbyId,
        user_id: UserId,
        message: String
    ) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            match_lobby_id,
            user_id,
            message,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn match_lobby_id(&self) -> MatchLobbyId { self.match_lobby_id }

    pub fn user_id(&self) -> UserId { self.user_id }

    pub fn message(&self) -> &str { &self.message }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbySendMessageResponse;

// Traits
impl ApiRequest for MatchLobbySendMessageRequest {
    type Response = MatchLobbySendMessageResponse;

    fn name() -> &'static str { "MatchLobbySendMessageRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_send_message"
    }
}

impl ApiResponse for MatchLobbySendMessageResponse {
    fn name() -> &'static str { "MatchLobbySendMessageResponse" }
}
