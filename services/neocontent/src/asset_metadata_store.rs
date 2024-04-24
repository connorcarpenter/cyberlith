use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

use asset_id::{AssetId, AssetType, ETag};
use logging::{info, warn};

use crate::error::AssetIoError;

pub struct AssetMetadata {
    path: String,
    asset_type: AssetType,
    etag: ETag,
    dependencies: Vec<AssetId>,
}

impl AssetMetadata {
    fn new(path: String, asset_type: AssetType, etag: ETag, dependencies: Vec<AssetId>) -> Self {
        Self {
            path,
            asset_type,
            etag,
            dependencies,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn asset_type(&self) -> AssetType {
        self.asset_type
    }

    pub fn etag(&self) -> ETag {
        self.etag
    }

    pub fn dependencies(&self) -> &Vec<AssetId> {
        &self.dependencies
    }
}

pub struct AssetMetadataStore {
    map: HashMap<AssetId, AssetMetadata>,
}

impl AssetMetadataStore {
    pub fn new(path: &str) -> Self {
        let mut output = Self {
            map: HashMap::new(),
        };

        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                warn!("Failed to read directory: {:?}, error: {:?}", path, e);
                panic!("path: {:?} should be created in `main()` in `local` environment, and by automation scripts for `prod`", path)
            }
        };

        for entry in entries {
            let Ok(entry) = entry else {
                continue;
            };
            let file_path = entry.path();

            // Check if the file has a .meta extension
            let Some(extension) = file_path.extension() else {
                continue;
            };
            if extension != "meta" {
                continue;
            }

            info!("Reading asset meta file: {:?}", file_path);
            let bytes = fs::read(&file_path).unwrap();

            let processed_meta = ProcessedAssetMeta::read(&bytes).unwrap();

            // strip ".meta" extension from file path
            let file_path_parent = file_path.parent().unwrap().to_str().unwrap();
            let file_name = file_path.file_stem().unwrap().to_str().unwrap();
            let asset_file_path = format!("{}/{}", file_path_parent, file_name);

            let new_file_extension = file_name.split('.').last().unwrap();
            let Some(asset_type) = AssetType::from_str(new_file_extension) else {
                panic!(
                    "Failed to find asset type for extension: {:?}",
                    new_file_extension
                )
            };

            output.insert(
                processed_meta.asset_id(),
                asset_type,
                processed_meta.etag(),
                processed_meta.dependencies(),
                asset_file_path,
            );
        }

        output
    }

    pub fn insert(
        &mut self,
        asset_id: AssetId,
        asset_type: AssetType,
        etag: ETag,
        dependencies: Vec<AssetId>,
        path: String,
    ) {
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        if self.map.contains_key(&asset_id) {
            panic!("asset already exists in map: {:?}", asset_id);
        }
        self.map.insert(
            asset_id,
            AssetMetadata::new(path, asset_type, etag, dependencies),
        );
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&AssetMetadata> {
        self.map.get(asset_id)
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
        hash: Vec<u8>,
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

    pub fn etag(&self) -> ETag {
        ETag::from_str(&self.etag).unwrap()
    }

    pub fn dependencies(&self) -> Vec<AssetId> {
        self.dependencies
            .iter()
            .map(|s| AssetId::from_str(s).unwrap())
            .collect()
    }

    pub fn write(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }

    pub fn read(bytes: &[u8]) -> Result<Self, AssetIoError> {
        serde_json::from_slice(bytes).map_err(|e| AssetIoError::Message(e.to_string()))
    }
}
