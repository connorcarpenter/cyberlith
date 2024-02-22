use std::{collections::HashMap, any::TypeId};

use bevy_ecs::{prelude::Resource, entity::Entity, system::{Commands, ResMut}};
use bevy_log::info;

use naia_serde::{BitWriter, Serde};

use game_engine::{
    filesystem::{ReadResult, FileSystemManager, TaskKey},
    session::{SessionClient, LoadAssetRequest, LoadAssetWithData, LoadAssetResponse},
    asset::{AssetId, AnimationData, AssetHandle, AssetManager, AssetType, IconData, MeshFile, ModelData, PaletteData, SceneData, SkeletonData, SkinData, TypedAssetId},
    world::{Main, Alt1},
    naia::ResponseSendKey,
};

use crate::app::{systems::scene::WalkAnimation, resources::asset_metadata_store::{AssetMetadataSerde, AssetMetadataStore}};

type AssetProcessorId = TypeId;

pub enum LoadAssetTask {
    HasResponse(ResponseSendKey<LoadAssetResponse>, LoadAssetResponse),
    HasFsTask(AssetId, AssetType, ResponseSendKey<LoadAssetResponse>, TaskKey<ReadResult>),
}

/// Stores asset data in RAM
#[derive(Resource)]
pub struct AssetStore {
    path: String,
    metadata_store: AssetMetadataStore,
    data_store: HashMap<AssetId, Vec<u8>>,
    asset_processor_map: HashMap<AssetProcessorId, Box<dyn AssetDeferredProcessor>>,
    // entry entity -> (ref entity -> asset processor id)
    entry_waitlist: HashMap<Entity, HashMap<Entity, AssetProcessorId>>,
    // asset id -> (ref entity -> asset processor id)
    ref_waitlist: HashMap<AssetId, HashMap<Entity, AssetProcessorId>>,
    //
    load_asset_tasks: Vec<LoadAssetTask>
}

impl AssetStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            metadata_store: AssetMetadataStore::new(path),
            data_store: HashMap::new(),
            asset_processor_map: HashMap::new(),
            entry_waitlist: HashMap::new(),
            ref_waitlist: HashMap::new(),
            load_asset_tasks: Vec::new(),
        }
    }

    // added as a system to App
    pub fn handle_load_asset_tasks(
        mut asset_store: ResMut<AssetStore>,
        mut commands: Commands,
        mut session_client: SessionClient,
        mut fs_manager: ResMut<FileSystemManager>,
        mut asset_manager: ResMut<AssetManager>,
    ) {
        let load_asset_tasks = std::mem::take(&mut asset_store.load_asset_tasks);
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
                            asset_store.handle_data_store_load_asset(&mut commands, &mut asset_manager, &asset_id, &asset_type, asset_bytes);

                            Some((response_send_key, LoadAssetResponse::loaded_non_modified_asset()))
                        }
                        Some(Err(e)) => {
                            panic!("error reading asset from disk: {:?}", e.to_string());
                        }
                        None => {
                            // still pending
                            asset_store.load_asset_tasks.push(LoadAssetTask::HasFsTask(asset_id, asset_type, response_send_key, fs_task_key));
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
        file_system_manager: &mut FileSystemManager,
        request: LoadAssetRequest,
        response_send_key: ResponseSendKey<LoadAssetResponse>
    ) {
        let asset_id = request.asset_id;
        let asset_etag = request.etag;

        let Some(metadata) = self.metadata_store.get(&asset_id) else {
            // client has no asset
            self.load_asset_tasks.push(LoadAssetTask::HasResponse(response_send_key, LoadAssetResponse::has_old_or_no_asset()));
            return;
        };
        if metadata.etag() != asset_etag {
            // client has old asset
            self.load_asset_tasks.push(LoadAssetTask::HasResponse(response_send_key, LoadAssetResponse::has_old_or_no_asset()));
            return;
        }

        // client has current asset in disk

        // make sure asset is not in memory
        if self.data_store.contains_key(&asset_id) {
            panic!("asset is in memory. session server should not be asking for it!");
        }

        // load asset into memory
        info!("loading asset into memory: {:?}", metadata.path());
        let fs_task_key = file_system_manager.read(metadata.path());
        self.load_asset_tasks.push(LoadAssetTask::HasFsTask(asset_id, metadata.asset_type(), response_send_key, fs_task_key));
        return;
    }

    pub fn handle_load_asset_with_data_message(&mut self, commands: &mut Commands, asset_manager: &mut AssetManager, message: LoadAssetWithData) {

        let asset_id = message.asset_id;
        let asset_etag = message.asset_etag;
        let asset_type = message.asset_type;
        let asset_data = message.asset_data;

        let asset_file_path = format!("{}/{}", self.path, asset_id.to_string());
        let asset_metadata_file_path = format!("{}.meta", &asset_file_path);

        // load asset data into disk
        info!("attempting to write asset data to disk: {:?}", asset_file_path);
        filesystem::write(&asset_file_path, &asset_data).unwrap();

        // load asset metadata into disk
        info!("attempting to write asset metadata to disk: {:?}", asset_metadata_file_path);
        let metadata_payload = AssetMetadataSerde::new(asset_etag, asset_type);
        let mut metadata_writer = BitWriter::new();
        metadata_payload.ser(&mut metadata_writer);
        let metadata_bytes = metadata_writer.to_bytes();
        filesystem::write(asset_metadata_file_path, metadata_bytes).unwrap();

        // load asset data into memory
        info!("loading asset into memory: {:?}", asset_file_path);
        self.handle_data_store_load_asset(commands, asset_manager, &asset_id, &asset_type, asset_data);

        // load asset metadata into memory
        self.metadata_store.insert(asset_id, asset_etag, asset_file_path, asset_type);
    }

    pub fn handle_data_store_load_asset(
        &mut self,
        commands: &mut Commands,
        asset_manager: &mut AssetManager,
        asset_id: &AssetId,
        asset_type: &AssetType,
        asset_data: Vec<u8>
    ) {
        self.data_store.insert(*asset_id, asset_data);

        asset_manager.load(&self.data_store, asset_id, asset_type);

        // process any refs waiting for this asset
        if let Some(ref_waitlist_entry) = self.ref_waitlist.remove(asset_id) {

            let typed_asset_id = TypedAssetId::new(*asset_id, *asset_type);

            for (entity, asset_processor_id) in ref_waitlist_entry {
                let asset_processor = self.asset_processor_map.remove(&asset_processor_id).unwrap();

                asset_processor.deferred_process(commands, &entity, &typed_asset_id);

                self.asset_processor_map.insert(asset_processor_id, asset_processor);
            }
        }
    }

    // entry entity is here, just not the component just yet ...
    pub fn handle_add_asset_entry_waitlist<T: AssetProcessor>(&mut self, ref_entity: &Entity, entry_entity: &Entity) {
        info!("entity ({:?}) received AssetRef from World Server! waiting on asset entry..", ref_entity);
        // initialize asset processor if needed
        let asset_processor_id = T::id();
        if !self.asset_processor_map.contains_key(&asset_processor_id) {
            self.asset_processor_map.insert(asset_processor_id, T::make_deferred_box());
        }

        if !self.entry_waitlist.contains_key(entry_entity) {
            self.entry_waitlist.insert(*entry_entity, HashMap::new());
        }
        let entry_waitlist_entry = self.entry_waitlist.get_mut(entry_entity).unwrap();
        entry_waitlist_entry.insert(*ref_entity, asset_processor_id);
    }

    pub fn handle_add_asset_entry(&mut self, commands: &mut Commands, entry_entity: &Entity, asset_id: &AssetId) {
        info!("entity ({:?}) received AssetEntry from World Server! (asset_id: {:?})", entry_entity, asset_id);
        // initialize asset processor if needed
        if let Some(waitlist_entry) = self.entry_waitlist.remove(entry_entity) {
            for (ref_entity, asset_processor_id) in waitlist_entry {
                info!("Processing waiting AssetRef: {:?}", ref_entity);

                // mirror logic in handle_entity_added_asset_ref
                if self.data_store.contains_key(asset_id) {
                    // process asset ref
                    let asset_processor = self.asset_processor_map.remove(&asset_processor_id).unwrap();

                    let asset_type = self.metadata_store.get(asset_id).unwrap().asset_type();
                    let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
                    asset_processor.deferred_process(commands, &ref_entity, &typed_asset_id);

                    self.asset_processor_map.insert(asset_processor_id, asset_processor);
                } else {
                    // put ref into waitlist
                    info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
                    if !self.ref_waitlist.contains_key(asset_id) {
                        self.ref_waitlist.insert(*asset_id, HashMap::new());
                    }
                    let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
                    ref_waitlist_entry.insert(ref_entity, asset_processor_id);
                }
            }
        }
    }

    pub fn handle_entity_added_asset_ref<T: AssetProcessor>(&mut self, commands: &mut Commands, ref_entity: &Entity, asset_id: &AssetId) {
        info!("entity ({:?}) received AssetRef from World Server! (asset_id: {:?})", ref_entity, asset_id);
        if self.data_store.contains_key(asset_id) {
            // process asset ref
            let asset_type = self.metadata_store.get(asset_id).unwrap().asset_type();
            let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
            T::process(commands, ref_entity, &typed_asset_id);
        } else {
            // initialize asset processor if needed
            let asset_processor_id = T::id();
            if !self.asset_processor_map.contains_key(&asset_processor_id) {
                self.asset_processor_map.insert(asset_processor_id, T::make_deferred_box());
            }

            // put ref into waitlist
            info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
            if !self.ref_waitlist.contains_key(asset_id) {
                self.ref_waitlist.insert(*asset_id, HashMap::new());
            }
            let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
            ref_waitlist_entry.insert(*ref_entity, asset_processor_id);
        }
    }
}

pub trait AssetProcessor: Send + Sync + 'static {
    fn id() -> AssetProcessorId;
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor>;
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId);
}

impl AssetDeferredProcessor for Main {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(commands, entity, typed_asset_id);
    }
}

impl AssetProcessor for Main {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        info!("processing for entity: {:?} = inserting AssetRef<Main>(asset_id: {:?}) ", entity, typed_asset_id.get_id());

        let asset_type = typed_asset_id.get_type();
        let asset_id = typed_asset_id.get_id();
        match asset_type {
            AssetType::Skeleton => process_type::<SkeletonData>(commands, entity, &asset_id),
            AssetType::Mesh => process_type::<MeshFile>(commands, entity, &asset_id),
            AssetType::Palette => process_type::<PaletteData>(commands, entity, &asset_id),
            AssetType::Animation => process_type::<AnimationData>(commands, entity, &asset_id),
            AssetType::Icon => process_type::<IconData>(commands, entity, &asset_id),
            AssetType::Skin => process_type::<SkinData>(commands, entity, &asset_id),
            AssetType::Model => process_type::<ModelData>(commands, entity, &asset_id),
            AssetType::Scene => process_type::<SceneData>(commands, entity, &asset_id),
        }
    }
}

impl AssetDeferredProcessor for Alt1 {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(commands, entity, typed_asset_id);
    }
}

impl AssetProcessor for Alt1 {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        info!("processing for entity: {:?} = inserting AssetRef<Alt1>(asset_id: {:?}) ", entity, typed_asset_id.get_id());

        let asset_type = typed_asset_id.get_type();
        let asset_id = typed_asset_id.get_id();
        match asset_type {
            AssetType::Animation => {
                let walk_anim = WalkAnimation::new(AssetHandle::<AnimationData>::new(asset_id));
                commands.entity(*entity).insert(walk_anim);
            }
            _ => { panic!("unsupported asset type: {:?}", asset_type); }
        }
    }
}

fn process_type<T: Send + Sync + 'static>(commands: &mut Commands, entity: &Entity, asset_id: &AssetId) {
    commands.entity(*entity).insert(AssetHandle::<T>::new(*asset_id));
}

pub trait AssetDeferredProcessor: Send + Sync + 'static {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId);
}