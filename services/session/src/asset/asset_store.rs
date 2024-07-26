use std::collections::{HashMap, HashSet};

use asset_id::{AssetId, AssetType, ETag};
use logging::warn;

struct AssetData {
    asset_type: AssetType,
    etag: ETag,
    dependencies: HashSet<AssetId>,
    data: Vec<u8>,
}

impl AssetData {
    pub fn new(
        asset_type: AssetType,
        etag: ETag,
        dependencies: HashSet<AssetId>,
        data: Vec<u8>,
    ) -> Self {
        Self {
            asset_type,
            etag,
            dependencies,
            data,
        }
    }

    pub fn byte_count(&self) -> usize {
        self.data.len()
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

    pub fn insert_data(
        &mut self,
        asset_id: AssetId,
        asset_type: AssetType,
        etag: ETag,
        dependencies: HashSet<AssetId>,
        data: Vec<u8>,
    ) {
        if self.map.contains_key(&asset_id) {
            // this can sometimes happen when 2 assets are requested at the same time
            warn!(
                "attempted to insert asset w/ id {:?} which is already in memory",
                asset_id
            );
        } else {
            // info!("inserting asset w/ id: {:?}", asset_id);
            self.map.insert(
                asset_id,
                AssetData::new(asset_type, etag, dependencies, data),
            );
        }
    }

    pub fn has_asset(&self, asset_id: &AssetId) -> bool {
        self.map.contains_key(asset_id)
    }

    pub fn get_etag(&self, asset_id: &AssetId) -> Option<ETag> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some(asset_data.etag),
            None => None,
        }
    }

    pub fn get_dependencies(&self, asset_id: &AssetId) -> Option<&HashSet<AssetId>> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some(&asset_data.dependencies),
            None => None,
        }
    }

    pub fn get_type_and_etag_and_data(
        &self,
        asset_id: &AssetId,
    ) -> Option<(AssetType, ETag, Vec<u8>)> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some((
                asset_data.asset_type,
                asset_data.etag,
                asset_data.data.clone(),
            )),
            None => None,
        }
    }

    pub fn get_size_bytes(&self, asset_id: &AssetId) -> Option<usize> {
        match self.map.get(asset_id) {
            Some(asset_data) => Some(asset_data.byte_count()),
            None => None,
        }
    }
}
