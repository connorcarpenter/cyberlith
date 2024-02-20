use naia_bevy_shared::Message;

use asset_id::{AssetId, ETag};

#[derive(Message)]
pub struct AssetDataMessage {
    pub asset_id: AssetId,
    pub asset_etag: ETag,
    pub asset_data: Vec<u8>,
}

impl AssetDataMessage {
    pub fn new(asset_id: AssetId, asset_etag: ETag, asset_data: Vec<u8>) -> Self {
        Self {
            asset_id,
            asset_etag,
            asset_data,
        }
    }
}
