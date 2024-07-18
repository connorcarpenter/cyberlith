use naia_serde::SerdeInternal as Serde;

use auth_server_types::UserId;
use bevy_http_shared::{ApiRequest, ApiResponse, Method};
use social_server_types::{LobbyId, MessageId, Timestamp};

#[derive(Serde, PartialEq, Clone)]
pub enum SocialLobbyPatch {
    // lobby id, lobby name, owner user id
    Create(LobbyId, String, UserId),
    // lobby id, joining user id
    Join(LobbyId, UserId),
    // leaving user id
    Leave(UserId),
    // message id, timestamp, user id, message
    Message(MessageId, Timestamp, UserId, String),
    // lobby id
    Start(LobbyId),
}

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbiesRequest {
    social_secret: String,
    patches: Vec<SocialLobbyPatch>,
}

impl SocialPatchMatchLobbiesRequest {
    pub fn new(social_secret: &str, lobby_patches: Vec<SocialLobbyPatch>) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            patches: lobby_patches,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn patches(&self) -> &Vec<SocialLobbyPatch> {
        &self.patches
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchMatchLobbiesResponse;

// Traits
impl ApiRequest for SocialPatchMatchLobbiesRequest {
    type Response = SocialPatchMatchLobbiesResponse;

    fn name() -> &'static str {
        "SocialPatchMatchLobbiesRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_match_lobbies"
    }
}

impl ApiResponse for SocialPatchMatchLobbiesResponse {
    fn name() -> &'static str {
        "SocialPatchMatchLobbiesResponse"
    }
}
