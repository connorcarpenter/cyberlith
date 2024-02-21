use std::collections::HashMap;

use bevy_log::{info, warn};

use game_engine::{AssetId, ETag};

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
        let mut map = HashMap::new();

        let fs_read_dir_result = filesystem::read_dir(path);
        match fs_read_dir_result {
            Ok(entries) => {
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

                    info!("Reading asset metadata file: {:?}", file_path);
                    let bytes = filesystem::read(&file_path).unwrap();

                    let asset_etag_str = String::from_utf8(bytes).unwrap();
                    let asset_etag = ETag::from_str(&asset_etag_str).unwrap();

                    // strip ".meta" extension from file path
                    let file_path_parent = file_path.parent().unwrap().to_str().unwrap();
                    let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                    let asset_file_path = format!("{}/{}", file_path_parent, file_name);

                    let metadata = AssetMetadata::new(asset_file_path, asset_etag);
                    let asset_id = AssetId::from_str(file_name).unwrap();

                    map.insert(asset_id, metadata);
                }
            }
            Err(e) => {
                warn!("Failed to read directory: {:?}", e);

                info!("Creating directory: {:?}", path);
                filesystem::create_dir(path).unwrap();
            }
        }

        Self {
            map,
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
