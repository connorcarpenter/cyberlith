use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectAssetServerRequest {
    region_secret: String,
}

impl DisconnectAssetServerRequest {
    pub fn new(region_secret: &str) -> Self {
        Self {
            region_secret: region_secret.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectAssetServerResponse;

// Traits
impl ApiRequest for DisconnectAssetServerRequest {
    type Response = DisconnectAssetServerResponse;

    fn name() -> &'static str {
        "DisconnectAssetServerRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "disconnect_asset_server"
    }
}

impl ApiResponse for DisconnectAssetServerResponse {
    fn name() -> &'static str {
        "DisconnectAssetServerResponse"
    }
}
