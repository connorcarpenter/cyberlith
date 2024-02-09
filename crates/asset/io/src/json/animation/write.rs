use crypto::U32Token;

use crate::json::{AnimFile, Asset, AssetData, AssetMeta};

impl AnimFile {
    pub fn write(&self, asset_id: &U32Token) -> Box<[u8]> {
        let new_meta = AssetMeta::new(asset_id, AnimFile::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Animation(self.clone()));
        asset.write().into_boxed_slice()
    }
}