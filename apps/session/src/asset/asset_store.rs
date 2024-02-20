use asset_id::{AssetId, ETag};
use std::collections::HashMap;

struct AssetData {
    etag: ETag,
    data: Vec<u8>,
}

impl AssetData {
    pub fn new(etag: ETag, data: Vec<u8>) -> Self {
        Self { etag, data }
    }
}

/// Stores asset data and etags in RAM
pub struct AssetStore {
    map: HashMap<AssetId, AssetData>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert_data(&mut self, asset_id: AssetId, etag: ETag, data: Vec<u8>) {
        self.map.insert(asset_id, AssetData::new(etag, data));
    }

    pub fn get_etag(&self, asset_id: &AssetId) -> Option<ETag> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some(asset_data.etag),
            None => None,
        }
    }

    pub fn get_etag_and_data(&self, asset_id: &AssetId) -> Option<(ETag, Vec<u8>)> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some((asset_data.etag, asset_data.data.clone())),
            None => None,
        }
    }
}
