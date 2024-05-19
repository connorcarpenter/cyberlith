use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct ConnectSocialServerRequest {
    region_secret: String,
    http_addr: String,
    http_port: u16,
}

impl ConnectSocialServerRequest {
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
pub struct ConnectSocialServerResponse;

// Traits
impl ApiRequest for ConnectSocialServerRequest {
    type Response = ConnectSocialServerResponse;

    fn name() -> &'static str {
        "ConnectSocialServerRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "connect_social_server"
    }
}

impl ApiResponse for ConnectSocialServerResponse {
    fn name() -> &'static str {
        "ConnectSocialServerResponse"
    }
}
