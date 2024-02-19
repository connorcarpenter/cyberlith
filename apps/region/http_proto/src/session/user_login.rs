
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionUserLoginRequest {
    orchestrator_secret: String,
    // TODO: shouldn't send username & password in plaintext here
    pub username: String,
    pub password: String,
}

impl SessionUserLoginRequest {
    pub fn new(orchestrator_secret: &str, username: &str, password: &str) -> Self {
        Self {
            orchestrator_secret: orchestrator_secret.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn orchestrator_secret(&self) -> &str {
        &self.orchestrator_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SessionUserLoginResponse {
    pub session_server_public_webrtc_url: String,
    pub token: String,
}

impl SessionUserLoginResponse {
    pub fn new(session_server_public_webrtc_url: &str, token: &str) -> Self {
        Self {
            session_server_public_webrtc_url: session_server_public_webrtc_url.to_string(),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for SessionUserLoginRequest {
    type Response = SessionUserLoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "session/user_login"
    }
}

impl ApiResponse for SessionUserLoginResponse {}
