use naia_bevy_shared::{Message, Request, Response, Serde};

use asset_id::{AssetId, ETag};

#[derive(Message, Debug)]
pub struct LoadAssetRequest {
    pub asset_id: AssetId,
    pub etag: ETag,
}

impl Request for LoadAssetRequest {
    type Response = LoadAssetResponse;
}

impl LoadAssetRequest {
    pub fn new(asset_id: &AssetId, etag: &ETag) -> Self {
        Self {
            asset_id: *asset_id,
            etag: *etag,
        }
    }

    pub fn name() -> &'static str {
        "LoadAssetRequest"
    }
}

#[derive(Serde, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum LoadAssetResponseValue {
    ClientHasOldOrNoAsset,
    ClientLoadedNonModifiedAsset,
}

#[derive(Message, Eq, PartialEq, Hash, Debug)]
pub struct LoadAssetResponse {
    pub value: LoadAssetResponseValue,
}

impl Response for LoadAssetResponse {}

impl LoadAssetResponse {

    pub fn name() -> &'static str {
        "LoadAssetResponse"
    }

    pub fn has_old_or_no_asset() -> Self {
        Self {
            value: LoadAssetResponseValue::ClientHasOldOrNoAsset,
        }
    }

    pub fn loaded_non_modified_asset() -> Self {
        Self {
            value: LoadAssetResponseValue::ClientLoadedNonModifiedAsset,
        }
    }
}
