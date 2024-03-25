use bevy_ecs::{event::EventWriter, prelude::Resource, system::ResMut};
use bevy_log::info;

use naia_bevy_client::{Client, ResponseSendKey};

use session_server_naia_proto::messages::{LoadAssetRequest, LoadAssetResponse};

use asset_id::{AssetId, AssetType};
use asset_render::{AssetManager, AssetMetadataStore};
use filesystem::{FileSystemManager, ReadResult, TaskKey};

use crate::asset_cache::{AssetCache, AssetLoadedEvent};

type SessionClient<'a> = Client<'a, Session>;

use crate::networked::client_markers::Session;

#[cfg(feature = "networked")]
pub enum LoadAssetTask {
    HasResponse(ResponseSendKey<LoadAssetResponse>, LoadAssetResponse),
    HasFsTask(
        AssetId,
        AssetType,
        ResponseSendKey<LoadAssetResponse>,
        TaskKey<ReadResult>,
    ),
}

#[derive(Resource, Default)]
pub struct AssetCacheChecker {
    load_asset_tasks: Vec<LoadAssetTask>,
}

impl AssetCacheChecker {
    // added as a system to App
    pub fn handle_load_asset_tasks(
        mut asset_cache: ResMut<AssetCache>,
        mut asset_cache_checker: ResMut<AssetCacheChecker>,
        mut session_client: SessionClient,
        mut fs_manager: ResMut<FileSystemManager>,
        mut asset_manager: ResMut<AssetManager>,
        mut asset_loaded_event_writer: EventWriter<AssetLoadedEvent>,
    ) {
        let load_asset_tasks = std::mem::take(&mut asset_cache_checker.load_asset_tasks);
        // process load asset tasks
        for task in load_asset_tasks {
            let response_opt = match task {
                LoadAssetTask::HasResponse(response_send_key, response) => {
                    // already have response
                    Some((response_send_key, response))
                }
                LoadAssetTask::HasFsTask(asset_id, asset_type, response_send_key, fs_task_key) => {
                    match fs_manager.get_result(&fs_task_key) {
                        Some(Ok(result)) => {
                            let asset_bytes = result.bytes;
                            asset_cache.handle_data_store_load_asset(
                                &mut asset_manager,
                                &mut asset_loaded_event_writer,
                                &asset_id,
                                &asset_type,
                                asset_bytes,
                            );

                            Some((
                                response_send_key,
                                LoadAssetResponse::loaded_non_modified_asset(),
                            ))
                        }
                        Some(Err(e)) => {
                            panic!("error reading asset from disk: {:?}", e.to_string());
                        }
                        None => {
                            // still pending
                            asset_cache_checker
                                .load_asset_tasks
                                .push(LoadAssetTask::HasFsTask(
                                    asset_id,
                                    asset_type,
                                    response_send_key,
                                    fs_task_key,
                                ));
                            None
                        }
                    }
                }
            };
            if let Some((response_send_key, response)) = response_opt {
                session_client.send_response(&response_send_key, &response);
            }
        }
    }

    pub fn handle_load_asset_request(
        &mut self,
        asset_cache: &AssetCache,
        file_system_manager: &mut FileSystemManager,
        metadata_store: &mut AssetMetadataStore,
        request: LoadAssetRequest,
        response_send_key: ResponseSendKey<LoadAssetResponse>,
    ) {
        let asset_id = request.asset_id;
        let asset_etag = request.etag;

        let Some(metadata) = metadata_store.get(&asset_id) else {
            // client has no asset
            self.load_asset_tasks.push(LoadAssetTask::HasResponse(
                response_send_key,
                LoadAssetResponse::has_old_or_no_asset(),
            ));
            return;
        };
        if metadata.etag() != asset_etag {
            // client has old asset
            self.load_asset_tasks.push(LoadAssetTask::HasResponse(
                response_send_key,
                LoadAssetResponse::has_old_or_no_asset(),
            ));
            return;
        }

        // client has current asset in disk

        // make sure asset is not in memory
        if asset_cache.has_asset(&asset_id) {
            panic!("asset is in memory. session server should not be asking for it!");
        }

        // load asset into memory
        // info!("loading asset into memory: {:?}", metadata.path());
        let fs_task_key = file_system_manager.read(metadata.path());
        self.load_asset_tasks.push(LoadAssetTask::HasFsTask(
            asset_id,
            metadata.asset_type(),
            response_send_key,
            fs_task_key,
        ));
        return;
    }
}
