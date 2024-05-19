use naia_serde::SerdeInternal as Serde;
use http_common::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyDestroyRequest {
    session_secret: String,
    match_lobby_id: MatchLobbyId,
}

impl MatchLobbyDestroyRequest {
    pub fn new(session_secret: &str, match_lobby_id: MatchLobbyId) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            match_lobby_id,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }

    pub fn match_lobby_id(&self) -> MatchLobbyId { self.match_lobby_id }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct MatchLobbyDestroyResponse;

// Traits
impl ApiRequest for MatchLobbyDestroyRequest {
    type Response = MatchLobbyDestroyResponse;

    fn name() -> &'static str { "MatchLobbyDestroyRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "match_lobby_destroy"
    }
}

impl ApiResponse for MatchLobbyDestroyResponse {
    fn name() -> &'static str { "MatchLobbyDestroyResponse" }
}
