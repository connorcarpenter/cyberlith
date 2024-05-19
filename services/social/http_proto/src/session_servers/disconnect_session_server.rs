use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectSessionServerRequest {
    region_secret: String,
    http_addr: String,
    http_port: u16,
}

impl DisconnectSessionServerRequest {
    pub fn new(region_secret: &str, http_addr: &str, http_port: u16) -> Self {
        Self {
            region_secret: region_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }

    pub fn http_addr(&self) -> &str {
        &self.http_addr
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectSessionServerResponse;

// Traits
impl ApiRequest for DisconnectSessionServerRequest {
    type Response = DisconnectSessionServerResponse;

    fn name() -> &'static str { "DisconnectSessionServerRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "disconnect_session_server"
    }
}

impl ApiResponse for DisconnectSessionServerResponse {
    fn name() -> &'static str { "DisconnectSessionServerResponse" }
}
