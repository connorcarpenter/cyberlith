use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

use session_server_naia_proto::messages::AuthInner as SessionAuth;

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
    pub session_auth: SessionAuth,
}

impl SessionConnectResponse {
    pub fn new(token: &str) -> Self {
        Self {
            session_auth: SessionAuth::new(token),
        }
    }
}

// Traits
impl ApiRequest for SessionConnectRequest {
    type Response = SessionConnectResponse;

    fn name() -> &'static str {
        "SessionConnectRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "session/connect"
    }
}

impl ApiResponse for SessionConnectResponse {
    fn name() -> &'static str {
        "SessionConnectResponse"
    }
}
