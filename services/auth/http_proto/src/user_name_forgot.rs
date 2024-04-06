use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserNameForgotRequest {
    gateway_secret: String,

    pub email: String,
}

impl UserNameForgotRequest {
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
pub struct UserNameForgotResponse;

impl UserNameForgotResponse {
    pub fn new() -> Self { Self }
}

// Traits
impl ApiRequest for UserNameForgotRequest {
    type Response = UserNameForgotResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_name_forgot"
    }
}

impl ApiResponse for UserNameForgotResponse {}
