use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct RefreshTokenGrantRequest {
    pub refresh_token: String,
}

impl RefreshTokenGrantRequest {
    pub fn new(refresh_token: &str) -> Self {
        Self {
            refresh_token: refresh_token.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct RefreshTokenGrantResponse {
    pub access_token: String,
}

impl RefreshTokenGrantResponse {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for RefreshTokenGrantRequest {
    type Response = RefreshTokenGrantResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "refresh_token_grant"
    }
}

impl ApiResponse for RefreshTokenGrantResponse {}
