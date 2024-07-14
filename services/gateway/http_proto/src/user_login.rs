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
    simultaneous_login_detected: bool,
}

impl UserLoginResponse {
    pub fn success() -> Self {
        Self {
            simultaneous_login_detected: false,
        }
    }

    pub fn simultaneous_login_detected() -> Self {
        Self {
            simultaneous_login_detected: true,
        }
    }

    pub fn is_simultaneous_login_detected(&self) -> bool {
        self.simultaneous_login_detected
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
