use std::collections::HashMap;

use bevy_ecs::{prelude::Resource, entity::Entity};
use bevy_log::info;

use game_engine::{
    session::{LoadAssetRequest, LoadAssetWithData, LoadAssetResponse},
    asset::AssetId,
};

use crate::app::resources::asset_metadata_store::AssetMetadataStore;

/// Stores asset data in RAM
#[derive(Resource)]
pub struct AssetStore {
    path: String,
    metadata_store: AssetMetadataStore,
    data_store: HashMap<AssetId, Vec<u8>>,
}

impl AssetStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            metadata_store: AssetMetadataStore::new(path),
            data_store: HashMap::new(),
        }
    }

    pub fn handle_load_asset_request(&mut self, request: LoadAssetRequest) -> LoadAssetResponse {

        // TODO: currently this will ALWAYS return 'not found' because we don't add any assets to the cache

        let asset_id = request.asset_id;
        let asset_etag = request.etag;

        let Some(metadata) = self.metadata_store.get(&asset_id) else {
            // client has no asset
            return LoadAssetResponse::has_old_or_no_asset();
        };
        if metadata.etag() != asset_etag {
            // client has old asset
            return LoadAssetResponse::has_old_or_no_asset();
        }

        // client has current asset in disk

        // make sure asset is not in memory
        if self.data_store.contains_key(&asset_id) {
            panic!("asset is in memory. session server should not be asking for it!");
        }

        // load asset into memory
        info!("loading asset into memory: {:?}", metadata.path());
        let asset_bytes = filesystem::read(metadata.path()).unwrap();
        self.data_store.insert(asset_id, asset_bytes);

        return LoadAssetResponse::loaded_non_modified_asset();
    }

    pub fn handle_load_asset_with_data_message(&mut self, message: LoadAssetWithData) {

        let asset_id = message.asset_id;
        let asset_etag = message.asset_etag;
        let asset_data = message.asset_data;

        let asset_file_path = format!("{}/{}", self.path, asset_id.to_string());
        let asset_metadata_file_path = format!("{}.meta", &asset_file_path);

        // load asset data into disk
        info!("attempting to write asset data to disk: {:?}", asset_file_path);
        filesystem::write(&asset_file_path, &asset_data).unwrap();

        // load asset metadata into disk
        info!("attempting to write asset metadata to disk: {:?}", asset_metadata_file_path);
        filesystem::write(asset_metadata_file_path, asset_etag.to_string().as_bytes()).unwrap();

        // load asset data into memory
        info!("loading asset into memory: {:?}", asset_file_path);
        self.data_store.insert(asset_id, asset_data);

        // load asset metadata into memory
        self.metadata_store.insert(asset_id, asset_etag, asset_file_path);
    }

    pub fn handle_entity_added_asset_ref<T: Send + Sync + 'static>(&mut self, entity: &Entity, asset_id: &AssetId) {
        info!("entity ({:?}) received AssetRef from World Server! (asset_id: {:?})", entity, asset_id);
    }
}
