use std::{collections::HashMap, fs};

use log::info;

use asset_id::{AssetId, ETag};
use asset_io::json::ProcessedAssetMeta;

pub struct AssetMetadata {
    path: String,
    etag: ETag,
}

impl AssetMetadata {
    fn new(path: String, etag: ETag) -> Self {
        Self { path, etag }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn etag(&self) -> ETag {
        self.etag
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
            let file_path_parent = file_path.parent().unwrap().to_str().unwrap();
            let file_name = file_path.file_stem().unwrap().to_str().unwrap();
            let asset_file_path = format!("{}/{}", file_path_parent, file_name);

            output.insert(
                processed_meta.asset_id(),
                processed_meta.etag(),
                asset_file_path,
            );
        }

        output
    }

    pub fn insert(&mut self, asset_id: AssetId, etag: ETag, path: String) {
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        self.map.insert(asset_id, AssetMetadata::new(path, etag));
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&AssetMetadata> {
        self.map.get(asset_id)
    }
}
