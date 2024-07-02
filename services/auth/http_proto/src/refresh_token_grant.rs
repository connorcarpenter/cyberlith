use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

use crate::{AccessToken, RefreshToken};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct RefreshTokenGrantRequest {
    pub refresh_token: RefreshToken,
}

impl RefreshTokenGrantRequest {
    pub fn new(refresh_token: RefreshToken) -> Self {
        Self { refresh_token }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct RefreshTokenGrantResponse {
    pub user_id: UserId,
    pub user_name: String,
    pub access_token: AccessToken,
}

impl RefreshTokenGrantResponse {
    pub fn new(user_id: UserId, user_name: String, access_token: AccessToken) -> Self {
        Self {
            user_id,
            user_name,
            access_token,
        }
    }
}

// Traits
impl ApiRequest for RefreshTokenGrantRequest {
    type Response = RefreshTokenGrantResponse;

    fn name() -> &'static str {
        "RefreshTokenGrantRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "refresh_token_grant"
    }
}

impl ApiResponse for RefreshTokenGrantResponse {
    fn name() -> &'static str {
        "RefreshTokenGrantResponse"
    }
}
