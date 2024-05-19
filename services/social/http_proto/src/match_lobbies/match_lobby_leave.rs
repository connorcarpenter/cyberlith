use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyLeaveRequest {
    session_secret: String,
    user_id: UserId,
}

impl MatchLobbyLeaveRequest {
    pub fn new(session_secret: &str, user_id: UserId) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            user_id,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn user_id(&self) -> UserId { self.user_id }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyLeaveResponse;

// Traits
impl ApiRequest for MatchLobbyLeaveRequest {
    type Response = MatchLobbyLeaveResponse;

    fn name() -> &'static str { "MatchLobbyLeaveRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_leave"
    }
}

impl ApiResponse for MatchLobbyLeaveResponse {
    fn name() -> &'static str { "MatchLobbyLeaveResponse" }
}
