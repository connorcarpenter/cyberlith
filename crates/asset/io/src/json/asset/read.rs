use serde::{Deserialize, Serialize};

use crate::{json::{Asset, AssetMeta}, AssetIoError, AssetId, ETag};

impl Asset {
    pub fn read(bytes: &[u8]) -> Result<Self, AssetIoError> {
        serde_json::from_slice(bytes).map_err(|e| AssetIoError::Message(e.to_string()))
    }
}

impl AssetMeta {
    pub fn read_from_file(bytes: &[u8]) -> Result<(Self, String), AssetIoError> {
        let (meta, data) = Asset::read(bytes)?.deconstruct();
        return Ok((meta, data.type_name()));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessedAssetMeta {
    asset_id: String,
    etag: String,
    schema_version: u32,
    dependencies: Vec<String>,
    hash: Vec<u8>,
}

impl ProcessedAssetMeta {
    pub fn new(
        asset_id: AssetId,
        etag: ETag,
        schema_version: u32,
        dependencies: Vec<AssetId>,
        hash: Vec<u8>
    ) -> Self {
        let dependencies = dependencies.into_iter().map(|id| id.as_string()).collect();
        Self {
            asset_id: asset_id.as_string(),
            etag: etag.as_string(),
            schema_version,
            dependencies,
            hash,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        AssetId::from_str(&self.asset_id).unwrap()
    }

    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn write(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }

    pub fn read(bytes: &[u8]) -> Result<Self, AssetIoError> {
        serde_json::from_slice(bytes).map_err(|e| AssetIoError::Message(e.to_string()))
    }
}