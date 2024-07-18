use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;
use social_server_types::LobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyStartRequest {
    session_instance_secret: String,
    user_id: UserId,
}

impl MatchLobbyStartRequest {
    pub fn new(session_instance_secret: &str, user_id: UserId) -> Self {
        Self {
            session_instance_secret: session_instance_secret.to_string(),
            user_id,
        }
    }

    pub fn session_instance_secret(&self) -> &str {
        &self.session_instance_secret
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyStartResponse {
    lobby_id: LobbyId
}

impl MatchLobbyStartResponse {
    pub fn new(lobby_id: LobbyId) -> Self {
        Self {
            lobby_id
        }
    }

    pub fn lobby_id(&self) -> LobbyId {
        self.lobby_id
    }
}

// Traits
impl ApiRequest for MatchLobbyStartRequest {
    type Response = MatchLobbyStartResponse;

    fn name() -> &'static str {
        "MatchLobbyStartRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_start"
    }
}

impl ApiResponse for MatchLobbyStartResponse {
    fn name() -> &'static str {
        "MatchLobbyStartResponse"
    }
}
