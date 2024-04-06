use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmRequest {
    gateway_secret: String,

    pub register_token: String,
}

impl UserRegisterConfirmRequest {
    pub fn new(gateway_secret: &str, register_token: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            register_token: register_token.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterConfirmResponse {
    pub access_token: String,
}

impl UserRegisterConfirmResponse {
    pub fn new(access_token: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for UserRegisterConfirmRequest {
    type Response = UserRegisterConfirmResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_register_confirm"
    }
}

impl ApiResponse for UserRegisterConfirmResponse {}
