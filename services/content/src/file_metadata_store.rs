use std::{collections::HashMap, fs};

use naia_serde::SerdeInternal as Serde;

use asset_id::ETag;
use logging::{info, warn};

use automation_lib::ProcessedFileMeta;

// FileType
#[derive(Serde, Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub enum FileType {
    Html,
    Js,
    Wasm,
}

impl FileType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "html" => Some(Self::Html),
            "js" => Some(Self::Js),
            "wasm" => Some(Self::Wasm),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct FileMetadata {
    path: String,
    file_type: FileType,
    etag: ETag,
}

impl FileMetadata {
    fn new(path: String, file_type: FileType, etag: ETag) -> Self {
        Self {
            path,
            file_type,
            etag,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    pub fn etag(&self) -> ETag {
        self.etag
    }
}

pub struct FileMetadataStore {
    map: HashMap<String, FileMetadata>,
}

impl FileMetadataStore {
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

            info!("Reading content meta file: {:?}", file_path);
            let bytes = fs::read(&file_path).unwrap();

            let processed_meta = ProcessedFileMeta::read(&bytes).unwrap();

            // info!("meta file at {:?}, has ETag of: {:?}", file_path, processed_meta.etag());

            // strip ".meta" extension from file path
            let file_path_parent = file_path.parent().unwrap().to_str().unwrap();
            let file_name = file_path.file_stem().unwrap().to_str().unwrap();
            let file_path = format!("{}/{}", file_path_parent, file_name);

            let new_file_extension = file_name.split('.').last().unwrap();
            let Some(file_type) = FileType::from_str(new_file_extension) else {
                panic!(
                    "Failed to find file type for extension: {:?}",
                    new_file_extension
                )
            };

            output.insert(
                processed_meta.name(),
                file_type,
                processed_meta.etag(),
                file_path,
            );
        }

        output
    }

    pub fn insert(&mut self, name: String, file_type: FileType, etag: ETag, path: String) {
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        if self.map.contains_key(&name) {
            panic!("asset already exists in map: {:?}", name);
        }
        self.map
            .insert(name, FileMetadata::new(path, file_type, etag));
    }

    pub fn get(&self, name: &str) -> Option<&FileMetadata> {
        self.map.get(name)
    }
}
