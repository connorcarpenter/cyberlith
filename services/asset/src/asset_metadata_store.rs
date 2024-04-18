use std::{collections::HashMap, fs};

use logging::{info, warn};

use asset_id::{AssetId, AssetType, ETag};
use asset_serde::json::ProcessedAssetMeta;

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
