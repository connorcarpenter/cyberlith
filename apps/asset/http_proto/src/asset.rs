use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct AssetRequest {
    // TODO: secret?
    asset_id: String,
}

impl AssetRequest {
    pub fn new(asset_id: &str) -> Self {
        Self {
            asset_id: asset_id.to_string(),
        }
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct AssetResponse;

// Traits
impl ApiRequest for AssetRequest {
    type Response = AssetResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "asset"
    }
}

impl ApiResponse for AssetResponse {}
