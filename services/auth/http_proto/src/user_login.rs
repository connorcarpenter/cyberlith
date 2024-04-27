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
pub struct UserLoginResponse {
    pub refresh_token: String,
    pub access_token: String,
}

impl UserLoginResponse {
    pub fn new(refresh_token: &str, access_token: &str) -> Self {
        Self {
            refresh_token: refresh_token.to_string(),
            access_token: access_token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for UserLoginRequest {
    type Response = UserLoginResponse;

    fn name() -> &'static str { "UserLoginRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_login"
    }
}

impl ApiResponse for UserLoginResponse {
    fn name() -> &'static str { "UserLoginResponse" }
}
