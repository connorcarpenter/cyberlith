use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct AccessTokenValidateRequest {
    pub access_token: String,
}

impl AccessTokenValidateRequest {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct AccessTokenValidateResponse;

impl AccessTokenValidateResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for AccessTokenValidateRequest {
    type Response = AccessTokenValidateResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "access_token_validate"
    }
}

impl ApiResponse for AccessTokenValidateResponse {}
