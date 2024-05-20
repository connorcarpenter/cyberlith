use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectSessionServerRequest {
    region_secret: String,
    session_secret: String,
}

impl DisconnectSessionServerRequest {
    pub fn new(region_secret: &str, session_secret: &str) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            session_secret: session_secret.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectSessionServerResponse;

// Traits
impl ApiRequest for DisconnectSessionServerRequest {
    type Response = DisconnectSessionServerResponse;

    fn name() -> &'static str {
        "DisconnectSessionServerRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "disconnect_session_server"
    }
}

impl ApiResponse for DisconnectSessionServerResponse {
    fn name() -> &'static str {
        "DisconnectSessionServerResponse"
    }
}
