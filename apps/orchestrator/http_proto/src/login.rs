use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct LoginRequest {
    // TODO: shouldn't send username & password in plaintext here
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct LoginResponse {
    pub session_server_public_webrtc_url: String,
    pub token: String,
}

impl LoginResponse {
    pub fn new(session_server_public_webrtc_url: &str, token: &str) -> Self {
        Self {
            session_server_public_webrtc_url: session_server_public_webrtc_url.to_string(),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for LoginRequest {
    type Response = LoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "login"
    }
}

impl ApiResponse for LoginResponse {}
