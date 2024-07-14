use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

use auth_server_types::UserId;

#[derive(Serde, PartialEq, Clone)]
pub enum SocialUserPatch {
    Add(UserId),
    Remove(UserId),
}

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialPatchUsersRequest {
    social_secret: String,
    patches: Vec<SocialUserPatch>,
}

impl SocialPatchUsersRequest {
    pub fn new(social_secret: &str, patches: Vec<SocialUserPatch>) -> Self {
        Self {
            social_secret: social_secret.to_string(),
            patches,
        }
    }

    pub fn social_secret(&self) -> &str {
        &self.social_secret
    }

    pub fn user_patches(&self) -> &Vec<SocialUserPatch> {
        &self.patches
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
