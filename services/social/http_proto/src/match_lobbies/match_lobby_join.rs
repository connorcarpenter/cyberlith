use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyJoinRequest {
    session_secret: String,
    match_lobby_id: MatchLobbyId,
    user_id: UserId,
}

impl MatchLobbyJoinRequest {
    pub fn new(session_secret: &str, match_lobby_id: MatchLobbyId, user_id: UserId) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            match_lobby_id,
            user_id,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn match_lobby_id(&self) -> MatchLobbyId { self.match_lobby_id }

    pub fn user_id(&self) -> UserId { self.user_id }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyJoinResponse;

// Traits
impl ApiRequest for MatchLobbyJoinRequest {
    type Response = MatchLobbyJoinResponse;

    fn name() -> &'static str { "MatchLobbyJoinRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_join"
    }
}

impl ApiResponse for MatchLobbyJoinResponse {
    fn name() -> &'static str { "MatchLobbyJoinResponse" }
}
