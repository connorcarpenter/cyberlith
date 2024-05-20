use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::MatchLobbyId;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbyUsersRequest {
    social_secret: String,
    added_users: Vec<(MatchLobbyId, Vec<UserId>)>,
    removed_users: Vec<(MatchLobbyId, Vec<UserId>)>,
}

impl SocialPatchMatchLobbyUsersRequest {
    pub fn new(
        social_secret: &str,
        added_users: Vec<(MatchLobbyId, Vec<UserId>)>,
        removed_users: Vec<(MatchLobbyId, Vec<UserId>)>,
    ) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            added_users,
            removed_users,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn added_users(&self) -> &Vec<(MatchLobbyId, Vec<UserId>)> {
        &self.added_users
    }

    pub fn removed_users(&self) -> &Vec<(MatchLobbyId, Vec<UserId>)> {
        &self.removed_users
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbyUsersResponse;

// Traits
impl ApiRequest for SocialPatchMatchLobbyUsersRequest {
    type Response = SocialPatchMatchLobbyUsersResponse;

    fn name() -> &'static str {
        "SocialPatchMatchLobbyUsersRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_match_lobby_users"
    }
}

impl ApiResponse for SocialPatchMatchLobbyUsersResponse {
    fn name() -> &'static str {
        "SocialPatchMatchLobbyUsersResponse"
    }
}
