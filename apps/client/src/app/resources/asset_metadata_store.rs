use std::collections::HashMap;

use bevy_log::{info, warn};

use naia_serde::{SerdeInternal as Serde, BitReader};

use game_engine::asset::{AssetId, AssetType, ETag};


#[derive(Serde, Eq, PartialEq, Clone)]
pub struct AssetMetadataSerde {
    pub etag: ETag,
    pub asset_type: AssetType,
}

impl AssetMetadataSerde {
    pub fn new(etag: ETag, asset_type: AssetType) -> Self {
        Self { etag, asset_type }
    }
}

pub struct AssetMetadata {
    path: String,
    etag: ETag,
    asset_type: AssetType,
}

impl AssetMetadata {
    fn new(path: String, etag: ETag, asset_type: AssetType) -> Self {
        Self { path, etag, asset_type }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn etag(&self) -> ETag {
        self.etag
    }

    pub fn asset_type(&self) -> AssetType {
        self.asset_type
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
                    let metadata_bytes = filesystem::read(&file_path).unwrap();
                    let mut metadata_reader = BitReader::new(&metadata_bytes);
                    let metadata_payload = AssetMetadataSerde::de(&mut metadata_reader).unwrap();

                    let asset_etag = metadata_payload.etag;
                    let asset_type = metadata_payload.asset_type;

                    // strip ".meta" extension from file path
                    let file_path_parent = file_path.parent().unwrap().to_str().unwrap();
                    let file_name = file_path.file_stem().unwrap().to_str().unwrap();
                    let asset_file_path = format!("{}/{}", file_path_parent, file_name);

                    let metadata = AssetMetadata::new(asset_file_path, asset_etag, asset_type);
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

    pub fn insert(&mut self, asset_id: AssetId, etag: ETag, path: String, asset_type: AssetType) {
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        self.map.insert(asset_id, AssetMetadata::new(path, etag, asset_type));
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&AssetMetadata> {
        self.map.get(asset_id)
    }
}
