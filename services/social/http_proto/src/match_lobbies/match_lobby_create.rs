use auth_server_types::UserId;
use http_common::{ApiRequest, ApiResponse, Method};
use naia_serde::SerdeInternal as Serde;
use social_server_types::LobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyCreateRequest {
    session_instance_secret: String,
    creator_user_id: UserId,
    match_name: String,
}

impl MatchLobbyCreateRequest {
    pub fn new(session_secret: &str, creator_user_id: UserId, match_name: &str) -> Self {
        Self {
            session_instance_secret: session_secret.to_string(),
            creator_user_id,
            match_name: match_name.to_string(),
        }
    }

    pub fn session_instance_secret(&self) -> &str {
        &self.session_instance_secret
    }

    pub fn creator_user_id(&self) -> UserId {
        self.creator_user_id
    }

    pub fn match_name(&self) -> &str {
        &self.match_name
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyCreateResponse {
    match_lobby_id: LobbyId,
}

impl MatchLobbyCreateResponse {
    pub fn new(match_lobby_id: LobbyId) -> Self {
        Self { match_lobby_id }
    }

    pub fn match_lobby_id(&self) -> LobbyId {
        self.match_lobby_id
    }
}

// Traits
impl ApiRequest for MatchLobbyCreateRequest {
    type Response = MatchLobbyCreateResponse;

    fn name() -> &'static str {
        "MatchLobbyCreateRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_create"
    }
}

impl ApiResponse for MatchLobbyCreateResponse {
    fn name() -> &'static str {
        "MatchLobbyCreateResponse"
    }
}
