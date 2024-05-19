use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserLoginRequest {
    // username OR email
    pub handle: String,

    // password
    pub password: String,
}

impl UserLoginRequest {
    pub fn new(handle: &str, password: &str) -> Self {
        Self {
            handle: handle.to_string(),
            password: password.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct UserLoginResponse;

impl UserLoginResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for UserLoginRequest {
    type Response = UserLoginResponse;

    fn name() -> &'static str {
        "UserLoginRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "api/user_login"
    }
}

impl ApiResponse for UserLoginResponse {
    fn name() -> &'static str {
        "UserLoginResponse"
    }
}
