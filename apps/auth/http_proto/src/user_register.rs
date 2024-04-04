use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterRequest {
    gateway_secret: String,

    pub username: String,
    pub email: String,
    pub password: String,
}

impl UserRegisterRequest {
    pub fn new(gateway_secret: &str, username: &str, email: &str, password: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterResponse;

impl UserRegisterResponse {
    pub fn new() -> Self {
        Self {}
    }
}

// Traits
impl ApiRequest for UserRegisterRequest {
    type Response = UserRegisterResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_register"
    }
}

impl ApiResponse for UserRegisterResponse {}
