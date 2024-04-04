use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct TokenValidateRequest {
    gateway_secret: String,

    pub access_token: String,
}

impl TokenValidateRequest {
    pub fn new(gateway_secret: &str, access_token: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            access_token: access_token.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct TokenValidateResponse;

impl TokenValidateResponse {
    pub fn new() -> Self { Self }
}

// Traits
impl ApiRequest for TokenValidateRequest {
    type Response = TokenValidateResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "token_validate"
    }
}

impl ApiResponse for TokenValidateResponse {}
