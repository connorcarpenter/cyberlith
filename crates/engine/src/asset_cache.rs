use std::collections::HashMap;

use bevy_ecs::{
    event::{Event, EventWriter},
    prelude::Resource,
    system::ResMut,
};

use naia_serde::{BitWriter, Serde};

use asset_id::{AssetId, AssetType, ETag};
use asset_loader::{AssetManager, AssetMetadataSerde, AssetMetadataStore};
use filesystem::{FileSystemManager, TaskKey, WriteResult};
use ui_runner::UiManager;

/// Stores asset data in RAM
#[derive(Resource)]
pub struct AssetCache {
    path: String,
    data_store: HashMap<AssetId, Vec<u8>>,
    save_asset_tasks: Vec<SaveAssetTask>,
}

impl AssetCache {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            data_store: HashMap::new(),
            save_asset_tasks: Vec::new(),
        }
    }

    // added as a system to App
    pub fn handle_save_asset_tasks(
        mut asset_cache: ResMut<AssetCache>,
        mut fs_manager: ResMut<FileSystemManager>,
    ) {
        let save_asset_tasks = std::mem::take(&mut asset_cache.save_asset_tasks);
        for mut task in save_asset_tasks {
            task.process(&mut fs_manager);
            if !task.is_completed() {
                asset_cache.save_asset_tasks.push(task);
            }
        }
    }

    pub fn handle_load_asset_with_data_message(
        &mut self,
        asset_manager: &mut AssetManager,
        ui_manager: &mut UiManager,
        asset_loaded_event_writer: &mut EventWriter<AssetLoadedEvent>,
        file_system_manager: &mut FileSystemManager,
        metadata_store: &mut AssetMetadataStore,
        asset_id: AssetId,
        asset_etag: ETag,
        asset_type: AssetType,
        asset_data: Vec<u8>,
    ) {
        let asset_file_path = format!("{}/{}", self.path, asset_id.to_string());
        let asset_metadata_file_path = format!("{}.meta", &asset_file_path);

        // load asset data into disk
        // info!(
        //     "attempting to write asset data to disk: {:?}",
        //     asset_file_path
        // );
        let asset_write_key = file_system_manager.write(&asset_file_path, &asset_data);

        // load asset metadata into disk
        // info!(
        //     "attempting to write asset metadata to disk: {:?}",
        //     asset_metadata_file_path
        // );
        let metadata_payload = AssetMetadataSerde::new(asset_etag, asset_type);
        let mut metadata_writer = BitWriter::new();
        metadata_payload.ser(&mut metadata_writer);
        let metadata_bytes = metadata_writer.to_bytes();
        let metadata_write_key =
            file_system_manager.write(&asset_metadata_file_path, &metadata_bytes);

        // save write keys into task
        self.save_asset_tasks
            .push(SaveAssetTask::new(asset_write_key, metadata_write_key));

        // load asset data into memory
        // info!("loading asset into memory: {:?}", asset_file_path);
        self.handle_data_store_load_asset(
            asset_manager,
            ui_manager,
            asset_loaded_event_writer,
            &asset_id,
            &asset_type,
            asset_data,
        );

        // load asset metadata into memory
        metadata_store.insert(asset_id, asset_etag, asset_file_path, asset_type);
    }

    pub fn handle_data_store_load_asset(
        &mut self,
        asset_manager: &mut AssetManager,
        ui_manager: &mut UiManager,
        asset_loaded_event_writer: &mut EventWriter<AssetLoadedEvent>,
        asset_id: &AssetId,
        asset_type: &AssetType,
        asset_data: Vec<u8>,
    ) {
        if self.data_store.contains_key(asset_id) {
            panic!("asset is already in memory");
        }
        self.data_store.insert(*asset_id, asset_data);

        if asset_type == &AssetType::Ui {
            ui_manager.load(asset_manager, &self.data_store, asset_id);
        } else {
            asset_manager.load(&self.data_store, asset_id, asset_type);
        }

        asset_loaded_event_writer.send(AssetLoadedEvent {
            asset_id: *asset_id,
            asset_type: *asset_type,
        });
    }

    #[allow(unused)]
    pub fn has_asset(&self, asset_id: &AssetId) -> bool {
        self.data_store.contains_key(asset_id)
    }
}

#[derive(Event)]
pub struct AssetLoadedEvent {
    pub asset_id: AssetId,
    pub asset_type: AssetType,
}

pub struct SaveAssetTask {
    asset_write_key_opt: Option<TaskKey<WriteResult>>,
    metadata_write_key_opt: Option<TaskKey<WriteResult>>,
}

impl SaveAssetTask {
    pub fn new(
        asset_write_key: TaskKey<WriteResult>,
        metadata_write_key: TaskKey<WriteResult>,
    ) -> Self {
        Self {
            asset_write_key_opt: Some(asset_write_key),
            metadata_write_key_opt: Some(metadata_write_key),
        }
    }

    pub fn process(&mut self, fs_manager: &mut FileSystemManager) {
        if let Some(asset_write_key) = self.asset_write_key_opt {
            if let Some(result) = fs_manager.get_result(&asset_write_key) {
                match result {
                    Ok(_) => {
                        // info!("asset write completed");
                    }
                    Err(e) => {
                        panic!("error writing asset to disk: {:?}", e.to_string());
                    }
                }

                self.asset_write_key_opt = None;
            }
        }
        if let Some(metadata_write_key) = self.metadata_write_key_opt {
            if let Some(result) = fs_manager.get_result(&metadata_write_key) {
                match result {
                    Ok(_) => {
                        // info!("metadata write completed");
                    }
                    Err(e) => {
                        panic!("error writing metadata to disk: {:?}", e.to_string());
                    }
                }

                self.metadata_write_key_opt = None;
            }
        }
    }

    pub fn is_completed(&self) -> bool {
        self.asset_write_key_opt.is_none() && self.metadata_write_key_opt.is_none()
    }
}
