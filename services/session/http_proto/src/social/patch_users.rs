use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchUsersRequest {
    social_secret: String,
    added_users: Vec<UserId>,
    removed_users: Vec<UserId>,
}

impl SocialPatchUsersRequest {
    pub fn new(social_secret: &str, added_users: Vec<UserId>, removed_users: Vec<UserId>) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            added_users,
            removed_users,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn added_users(&self) -> &Vec<UserId> {
        &self.added_users
    }

    pub fn removed_users(&self) -> &Vec<UserId> {
        &self.removed_users
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchUsersResponse;

// Traits
impl ApiRequest for SocialPatchUsersRequest {
    type Response = SocialPatchUsersResponse;

    fn name() -> &'static str {
        "SocialPatchUsersRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social_patch_users"
    }
}

impl ApiResponse for SocialPatchUsersResponse {
    fn name() -> &'static str {
        "SocialPatchUsersResponse"
    }
}
