use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbyMessagesRequest {
    social_secret: String,
    new_messages: Vec<(MatchLobbyId, Vec<(UserId, String)>)>,
}

impl SocialPatchMatchLobbyMessagesRequest {
    pub fn new(
        social_secret: &str,
        new_messages: Vec<(MatchLobbyId, Vec<(UserId, String)>)>,
    ) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            new_messages,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn new_messages(&self) -> &Vec<(MatchLobbyId, Vec<(UserId, String)>)> {
        &self.new_messages
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbyMessagesResponse;

// Traits
impl ApiRequest for SocialPatchMatchLobbyMessagesRequest {
    type Response = SocialPatchMatchLobbyMessagesResponse;

    fn name() -> &'static str {
        "SocialPatchMatchLobbyMessagesRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_match_lobby_messages"
    }
}

impl ApiResponse for SocialPatchMatchLobbyMessagesResponse {
    fn name() -> &'static str {
        "SocialPatchMatchLobbyMessagesResponse"
    }
}
