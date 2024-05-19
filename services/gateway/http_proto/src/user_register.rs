use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserRegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl UserRegisterRequest {
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
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

    fn name() -> &'static str {
        "UserRegisterRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "api/user_register"
    }
}

impl ApiResponse for UserRegisterResponse {
    fn name() -> &'static str {
        "UserRegisterResponse"
    }
}
