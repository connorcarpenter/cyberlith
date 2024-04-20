use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionConnectRequest;

impl SessionConnectRequest {
    pub fn new() -> Self {
        Self
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SessionConnectResponse {
    pub session_server_public_webrtc_url: String,
    pub token: String,
}

impl SessionConnectResponse {
    pub fn new(session_server_public_webrtc_url: &str, token: &str) -> Self {
        Self {
            session_server_public_webrtc_url: session_server_public_webrtc_url.to_string(),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for SessionConnectRequest {
    type Response = SessionConnectResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "session/connect"
    }
}

impl ApiResponse for SessionConnectResponse {}
