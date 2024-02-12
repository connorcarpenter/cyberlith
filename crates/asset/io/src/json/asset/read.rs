use serde::{Deserialize, Serialize};

use crate::{json::{Asset, AssetMeta}, AssetIoError};
use crate::json::AssetId;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ProcessedAssetMeta {
    asset_id: String,
    asset_version: u32,
    schema_version: u32,
    dependencies: Vec<String>,
    hash: Vec<u8>,
}

impl ProcessedAssetMeta {
    pub fn new(
        asset_id: AssetId,
        asset_version: u32,
        schema_version: u32,
        dependencies: Vec<AssetId>,
        hash: Vec<u8>
    ) -> Self {
        let dependencies = dependencies.into_iter().map(|id| id.as_string()).collect();
        Self {
            asset_id: asset_id.as_string(),
            asset_version,
            schema_version,
            dependencies,
            hash,
        }
    }

    pub fn write(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }
}