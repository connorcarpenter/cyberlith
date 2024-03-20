use asset_id::AssetId;

use crate::json::{Asset, AssetData, AssetMeta, SceneJson};

impl SceneJson {
    pub fn write(&self, asset_id: &AssetId) -> Box<[u8]> {
        let new_meta = AssetMeta::new(asset_id, Self::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Scene(self.clone()));
        serde_json::to_vec_pretty(&asset)
            .unwrap()
            .into_boxed_slice()
    }
}
