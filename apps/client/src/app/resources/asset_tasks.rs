use bevy_log::info;

use game_engine::{asset::{AssetId, AssetType}, filesystem::{FileSystemManager, ReadResult, TaskKey, WriteResult}, naia::ResponseSendKey, session::LoadAssetResponse};

pub enum LoadAssetTask {
    HasResponse(ResponseSendKey<LoadAssetResponse>, LoadAssetResponse),
    HasFsTask(AssetId, AssetType, ResponseSendKey<LoadAssetResponse>, TaskKey<ReadResult>),
}

pub struct SaveAssetTask {
    asset_write_key_opt: Option<TaskKey<WriteResult>>,
    metadata_write_key_opt: Option<TaskKey<WriteResult>>,
}

impl SaveAssetTask {
    pub fn new(asset_write_key: TaskKey<WriteResult>, metadata_write_key: TaskKey<WriteResult>) -> Self {
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
                        info!("asset write completed");
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
                        info!("metadata write completed");
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