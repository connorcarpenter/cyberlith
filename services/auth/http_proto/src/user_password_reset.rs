use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordResetRequest {
    gateway_secret: String,

    pub reset_password_token: String,
    pub new_password: String,
}

impl UserPasswordResetRequest {
    pub fn new(gateway_secret: &str, reset_password_token: &str, new_password: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            reset_password_token: reset_password_token.to_string(),
            new_password: new_password.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordResetResponse;

impl UserPasswordResetResponse {
    pub fn new() -> Self { Self }
}

// Traits
impl ApiRequest for UserPasswordResetRequest {
    type Response = UserPasswordResetResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_password_reset"
    }
}

impl ApiResponse for UserPasswordResetResponse {}
