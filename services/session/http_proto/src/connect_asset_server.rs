use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct ConnectAssetServerRequest {
    region_secret: String,
    http_addr: String,
    http_port: u16,
}

impl ConnectAssetServerRequest {
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
pub struct ConnectAssetServerResponse;

// Traits
impl ApiRequest for ConnectAssetServerRequest {
    type Response = ConnectAssetServerResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "connect_asset_server"
    }
}

impl ApiResponse for ConnectAssetServerResponse {}
