use naia_serde::SerdeInternal as Serde;
use auth_server_types::UserId;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbiesRequest {
    social_secret: String,
    // match lobby id, match name, creator user id
    added_match_lobbies: Vec<(MatchLobbyId, String, UserId)>,
    removed_match_lobbies: Vec<MatchLobbyId>,
}

impl SocialPatchMatchLobbiesRequest {
    pub fn new(social_secret: &str, added_match_lobbies: Vec<(MatchLobbyId, String, UserId)>, removed_match_lobbies: Vec<MatchLobbyId>) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            added_match_lobbies,
            removed_match_lobbies,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn added_match_lobbies(&self) -> &Vec<(MatchLobbyId, String, UserId)> {
        &self.added_match_lobbies
    }

    pub fn removed_match_lobbies(&self) -> &Vec<MatchLobbyId> {
        &self.removed_match_lobbies
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbiesResponse;

// Traits
impl ApiRequest for SocialPatchMatchLobbiesRequest {
    type Response = SocialPatchMatchLobbiesResponse;

    fn name() -> &'static str { "SocialPatchMatchLobbiesRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_match_lobbies"
    }
}

impl ApiResponse for SocialPatchMatchLobbiesResponse {
    fn name() -> &'static str { "SocialPatchMatchLobbiesResponse" }
}
