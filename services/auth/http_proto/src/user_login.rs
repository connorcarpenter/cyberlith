use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct UserLoginRequest {
    gateway_secret: String,

    // username OR email
    pub handle: String,

    // password
    pub password: String,
}

impl UserLoginRequest {
    pub fn new(gateway_secret: &str, handle: &str, password: &str) -> Self {
        Self {
            gateway_secret: gateway_secret.to_string(),

            handle: handle.to_string(),
            password: password.to_string(),
        }
    }

    pub fn gateway_secret(&self) -> &str {
        &self.gateway_secret
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

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "user_login"
    }
}

impl ApiResponse for UserLoginResponse {}
