use std::{path::PathBuf, collections::HashMap};

use bevy_ecs::{change_detection::ResMut, system::Resource};
use bevy_log::{info, warn};

use naia_serde::{SerdeInternal as Serde, BitReader};

use game_engine::{filesystem::{CreateDirResult, ReadDirResult, FileSystemManager, ReadResult, TaskKey}, asset::{AssetId, AssetType, ETag}};

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

#[derive(Resource)]
pub struct AssetMetadataStore {
    path: String,
    finished_loading: bool,
    map: HashMap<AssetId, AssetMetadata>,
    tasks: Vec<AssetMetadataTask>,
}

impl AssetMetadataStore {
    pub fn new(path: &str) -> Self {

        Self {
            path: path.to_string(),
            map: HashMap::new(),
            finished_loading: false,
            tasks: Vec::new(),
        }
    }

    // added as a system to App
    pub fn startup(
        mut asset_metadata_store: ResMut<AssetMetadataStore>,
        mut fs_manager: ResMut<FileSystemManager>,
    ) {
        asset_metadata_store.load(&mut fs_manager);
    }

    // added as a system to App
    pub fn handle_metadata_tasks(
        mut asset_metadata_store: ResMut<AssetMetadataStore>,
        mut fs_manager: ResMut<FileSystemManager>,
    ) {
        asset_metadata_store.process_tasks(&mut fs_manager);
    }

    pub fn load(&mut self, fs_manager: &mut FileSystemManager) {
        self.tasks.push(AssetMetadataTask::load_dir(fs_manager, &self.path));
    }

    pub fn process_tasks(&mut self, fs_manager: &mut FileSystemManager) {

        if self.finished_loading {
            return;
        }

        let tasks = std::mem::take(&mut self.tasks);

        for task in tasks {
            match task {
                AssetMetadataTask::LoadDir(path, task_key) => {
                    match fs_manager.get_result(&task_key) {
                        Some(Ok(result)) => {
                            let entries = result.entries();
                            for entry in entries {

                                let file_path = entry.path();

                                // Check if the file has a .meta extension
                                let Some(extension) = file_path.extension() else {
                                    continue;
                                };
                                if extension != "meta" {
                                    continue;
                                }

                                info!("Reading asset metadata file: {:?}", file_path);
                                self.tasks.push(AssetMetadataTask::load_file(fs_manager, file_path.to_str().unwrap()));
                            }
                        }
                        Some(Err(err)) => {
                            warn!("Failed to read directory ({:?}): {:?}", path, err.to_string());

                            info!("Creating directory: {:?}", path);
                            self.tasks.push(AssetMetadataTask::create_dir(fs_manager, path));
                        }
                        None => {
                            self.tasks.push(AssetMetadataTask::LoadDir(path, task_key));
                        }
                    }
                }
                AssetMetadataTask::CreateDir(path, task_key) => {
                    match fs_manager.get_result(&task_key) {
                        Some(Ok(_result)) => {
                            info!("Created directory: {:?}", path);
                        }
                        Some(Err(err)) => {
                            panic!("Error creating directory ({:?}): {:?}", path, err.to_string());
                        }
                        None => {
                            self.tasks.push(AssetMetadataTask::CreateDir(path, task_key));
                        }
                    }
                }
                AssetMetadataTask::LoadFile(file_path, task_key) => {
                    match fs_manager.get_result(&task_key) {
                        Some(Ok(result)) => {
                            let metadata_bytes = result.bytes;
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

                            self.map.insert(asset_id, metadata);
                        }
                        Some(Err(err)) => {
                            panic!("Error reading file ({:?}): {:?}", file_path, err.to_string());
                        }
                        None => {
                            self.tasks.push(AssetMetadataTask::LoadFile(file_path, task_key));
                        }
                    }
                }
            }
        }

        if self.tasks.is_empty() {
            self.finished_loading = true;
            info!("Finished loading AssetMetadataStore");
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.finished_loading
    }

    pub fn get(&self, asset_id: &AssetId) -> Option<&AssetMetadata> {
        if !self.finished_loading {
            panic!("AssetMetadataStore is not finished loading");
        }
        self.map.get(asset_id)
    }

    pub fn insert(&mut self, asset_id: AssetId, etag: ETag, path: String, asset_type: AssetType) {
        if !self.finished_loading {
            panic!("AssetMetadataStore is not finished loading");
        }
        // info!("Inserting asset into map: asset_id: {:?}, etag: {:?}, path: {:?}", asset_id, etag, path);
        self.map.insert(asset_id, AssetMetadata::new(path, etag, asset_type));
    }
}

enum AssetMetadataTask {
    LoadDir(PathBuf, TaskKey<ReadDirResult>),
    LoadFile(PathBuf, TaskKey<ReadResult>),
    CreateDir(PathBuf, TaskKey<CreateDirResult>),
}

impl AssetMetadataTask {
    fn load_dir(fs_manager: &mut FileSystemManager, path: &str) -> Self {
        let task_key = fs_manager.read_dir(path);
        Self::LoadDir(PathBuf::from(path), task_key)
    }

    fn load_file(fs_manager: &mut FileSystemManager, path: &str) -> Self {
        let task_key = fs_manager.read(path);
        Self::LoadFile(PathBuf::from(path), task_key)
    }

    fn create_dir(fs_manager: &mut FileSystemManager, path: PathBuf) -> Self {
        let task_key = fs_manager.create_dir(path.clone());
        Self::CreateDir(path, task_key)
    }
}