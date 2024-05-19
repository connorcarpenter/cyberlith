use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct ConnectSessionServerRequest {
    region_secret: String,
    http_addr: String,
    http_port: u16,
}

impl ConnectSessionServerRequest {
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
pub struct ConnectSessionServerResponse;

// Traits
impl ApiRequest for ConnectSessionServerRequest {
    type Response = ConnectSessionServerResponse;

    fn name() -> &'static str {
        "ConnectSessionServerRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "connect_session_server"
    }
}

impl ApiResponse for ConnectSessionServerResponse {
    fn name() -> &'static str {
        "ConnectSessionServerResponse"
    }
}
