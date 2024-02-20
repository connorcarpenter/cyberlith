use naia_bevy_shared::{Message, Request, Response, Serde};

use asset_id::{AssetId, ETag};

#[derive(Message, Debug)]
pub struct AssetEtagRequest {
    pub asset_id: AssetId,
    pub etag: ETag,
}

impl Request for AssetEtagRequest {
    type Response = AssetEtagResponse;
}

impl AssetEtagRequest {
    pub fn new(asset_id: &AssetId, etag: &ETag) -> Self {
        Self {
            asset_id: *asset_id,
            etag: *etag,
        }
    }
}

#[derive(Serde, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AssetEtagResponseValue {
    ClientHasOldOrNoAsset,
    ClientLoadedNonModifiedAsset,
}

#[derive(Message, Eq, PartialEq, Hash, Debug)]
pub struct AssetEtagResponse {
    pub value: AssetEtagResponseValue,
}

impl Response for AssetEtagResponse {}

impl AssetEtagResponse {
    pub fn has_old_or_no_asset() -> Self {
        Self {
            value: AssetEtagResponseValue::ClientHasOldOrNoAsset,
        }
    }

    pub fn loaded_non_modified_asset() -> Self {
        Self {
            value: AssetEtagResponseValue::ClientLoadedNonModifiedAsset,
        }
    }
}
