use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct RefreshTokenGrantRequest {
    gateway_secret: String,

    pub refresh_token: String,
}

impl RefreshTokenGrantRequest {
    pub fn new(gateway_secret: &str, refresh_token: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            refresh_token: refresh_token.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
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
