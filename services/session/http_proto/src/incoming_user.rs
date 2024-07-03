use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct IncomingUserRequest {
    region_secret: String,
    pub user_id: UserId,
    pub login_token: String,
}

impl IncomingUserRequest {
    pub fn new(region_secret: &str, user_id: UserId, login_token: &str) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            user_id,
            login_token: login_token.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct IncomingUserResponse;

// Traits
impl ApiRequest for IncomingUserRequest {
    type Response = IncomingUserResponse;

    fn name() -> &'static str {
        "IncomingUserRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "incoming_user"
    }
}

impl ApiResponse for IncomingUserResponse {
    fn name() -> &'static str {
        "IncomingUserResponse"
    }
}
