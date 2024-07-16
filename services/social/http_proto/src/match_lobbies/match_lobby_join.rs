use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;
use social_server_types::LobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyJoinRequest {
    session_instance_secret: String,
    lobby_id: LobbyId,
    user_id: UserId,
}

impl MatchLobbyJoinRequest {
    pub fn new(session_secret: &str, lobby_id: LobbyId, user_id: UserId) -> Self {
        Self {
            session_instance_secret: session_secret.to_string(),
            lobby_id,
            user_id,
        }
    }

    pub fn session_instance_secret(&self) -> &str {
        &self.session_instance_secret
    }

    pub fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyJoinResponse;

// Traits
impl ApiRequest for MatchLobbyJoinRequest {
    type Response = MatchLobbyJoinResponse;

    fn name() -> &'static str {
        "MatchLobbyJoinRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_join"
    }
}

impl ApiResponse for MatchLobbyJoinResponse {
    fn name() -> &'static str {
        "MatchLobbyJoinResponse"
    }
}
