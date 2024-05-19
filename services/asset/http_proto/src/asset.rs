use naia_serde::SerdeInternal as Serde;

use asset_id::{AssetId, AssetType, ETag};
use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone, Debug)]
pub struct AssetRequest {
    // TODO: secret?
    asset_id: AssetId,
    etag_opt: Option<ETag>,
}

impl AssetRequest {
    pub fn new(asset_id: AssetId, etag_opt: Option<ETag>) -> Self {
        Self { asset_id, etag_opt }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    pub fn etag_opt(&self) -> Option<ETag> {
        self.etag_opt
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub enum AssetResponseValue {
    Modified(ETag, AssetType, Vec<AssetId>, Vec<u8>),
    NotModified,
}

#[derive(Serde, PartialEq, Clone)]
pub struct AssetResponse {
    pub value: AssetResponseValue,
}

impl AssetResponse {
    pub fn not_modified() -> Self {
        Self {
            value: AssetResponseValue::NotModified,
        }
    }

    pub fn modified(
        etag: ETag,
        asset_type: AssetType,
        dependencies: Vec<AssetId>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            value: AssetResponseValue::Modified(etag, asset_type, dependencies, data),
        }
    }
}

// Traits
impl ApiRequest for AssetRequest {
    type Response = AssetResponse;

    fn name() -> &'static str {
        "AssetRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "asset"
    }
}

impl ApiResponse for AssetResponse {
    fn name() -> &'static str {
        "AssetResponse"
    }
}
