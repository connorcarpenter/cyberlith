use std::collections::HashMap;

use bevy_ecs::prelude::Resource;

use game_engine::{
    session::{LoadAssetRequest, LoadAssetWithData, LoadAssetResponse},
    AssetId, ETag,
};

#[derive(Resource)]
pub struct AssetStore {
    map: HashMap<AssetId, ETag>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn handle_etag_request(&self, request: LoadAssetRequest) -> LoadAssetResponse {

        // TODO: currently this will ALWAYS return 'not found' because we don't add any assets to the cache

        let asset_id = request.asset_id;
        let asset_etag = request.etag;

        let Some(old_etag) = self.map.get(&asset_id) else {
            // client has no asset
            return LoadAssetResponse::has_old_or_no_asset();
        };
        if old_etag != &asset_etag {
            // client has old asset
            return LoadAssetResponse::has_old_or_no_asset();
        }
        // client has current asset

        // TODO: load asset into memory!
        todo!();

        return LoadAssetResponse::loaded_non_modified_asset();
    }

    pub fn handle_asset_data_message(&mut self, message: LoadAssetWithData) {

        let asset_id = message.asset_id;
        let asset_etag = message.asset_etag;
        let asset_data = message.asset_data;

        self.map.insert(asset_id, asset_etag);

        // TODO: load asset into memory!
        todo!();
    }
}
