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
    pub token: String,
}

impl SessionConnectResponse {
    pub fn new(token: &str) -> Self {
        Self {
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
