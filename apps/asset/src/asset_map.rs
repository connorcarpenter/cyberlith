use std::{collections::HashMap, fs};

use log::info;

use asset_id::{AssetId, ETag};
use asset_io::{json::ProcessedAssetMeta};

pub struct AssetMetadata {
    path: String,
    etag: ETag,
}

impl AssetMetadata {
    fn new(path: String, etag: ETag) -> Self {
        Self {
            path,
            etag,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn etag(&self) -> ETag {
        self.etag
    }
}

pub struct AssetMap {
    map: HashMap<AssetId, AssetMetadata>,
}

impl AssetMap {

    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, asset_id: AssetId, etag: ETag, path: String) {
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        self.map.insert(asset_id, AssetMetadata::new(path, etag));
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&AssetMetadata> {
        self.map.get(asset_id)
    }
}

pub fn init_asset_map(path: &str) -> AssetMap {

    let mut output = AssetMap::new();

    let entries = fs::read_dir(path).unwrap();

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
        let file_path = file_path.file_stem().unwrap();
        let new_file_path = file_path.to_string_lossy();

        output.insert(processed_meta.asset_id(), processed_meta.etag(), new_file_path.to_string());
    }

    output
}