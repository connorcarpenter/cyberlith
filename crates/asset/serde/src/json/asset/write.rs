use crate::json::{Asset, AssetData, AssetMeta};

impl Asset {
    pub fn new(meta: AssetMeta, data: AssetData) -> Self {
        Self { meta, data }
    }
}
