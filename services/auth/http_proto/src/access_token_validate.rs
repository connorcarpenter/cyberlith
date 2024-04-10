use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct AccessTokenValidateRequest {
    gateway_secret: String,

    pub access_token: String,
}

impl AccessTokenValidateRequest {
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
pub struct AccessTokenValidateResponse;

impl AccessTokenValidateResponse {
    pub fn new() -> Self { Self }
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
