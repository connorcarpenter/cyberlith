use std::collections::HashMap;
use bevy_ecs::prelude::Resource;

use game_engine::{AssetId, ETag, session::{AssetEtagRequest, AssetEtagResponse}};

#[derive(Resource)]
pub struct AssetCache {
    map: HashMap<AssetId, ETag>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn handle_etag_request(&self, request: AssetEtagRequest) -> AssetEtagResponse {
        // TODO: currently this will ALWAYS return 'not found' because we don't add any assets to the cache
        match self.map.get(&request.asset_id) {
            Some(etag) => AssetEtagResponse::found(*etag),
            None => AssetEtagResponse::not_found(),
        }
    }
}

