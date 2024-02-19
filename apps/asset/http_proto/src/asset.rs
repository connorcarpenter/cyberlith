use naia_serde::SerdeInternal as Serde;
use asset_id::{AssetId, ETag};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone, Debug)]
pub struct AssetRequest {
    // TODO: secret?
    asset_id: AssetId,
    etag: ETag,
}

impl AssetRequest {
    pub fn new(asset_id: AssetId, etag: ETag) -> Self {
        Self {
            asset_id,
            etag,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn etag(&self) -> ETag {
        self.etag
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub enum AssetResponseValue {
    AssetData(Vec<u8>),
    NotModified,
}

#[derive(Serde, PartialEq, Clone)]
pub struct AssetResponse {
    value: AssetResponseValue,
}

impl AssetResponse {
    pub fn not_modified() -> Self {
        Self {
            value: AssetResponseValue::NotModified,
        }
    }

    pub fn asset_data(data: Vec<u8>) -> Self {
        Self {
            value: AssetResponseValue::AssetData(data),
        }
    }
}

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
