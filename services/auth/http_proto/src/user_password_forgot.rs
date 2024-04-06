use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordForgotRequest {
    gateway_secret: String,

    pub email: String,
}

impl UserPasswordForgotRequest {
    pub fn new(gateway_secret: &str, email: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            email: email.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserPasswordForgotResponse;

impl UserPasswordForgotResponse {
    pub fn new() -> Self { Self }
}

// Traits
impl ApiRequest for UserPasswordForgotRequest {
    type Response = UserPasswordForgotResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_password_forgot"
    }
}

impl ApiResponse for UserPasswordForgotResponse {}
