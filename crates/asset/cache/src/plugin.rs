
use bevy_app::{App, Plugin, Update};

use crate::{embedded_asset::handle_embedded_asset_event, asset_cache::{AssetCache, AssetLoadedEvent}};

pub struct AssetCachePlugin;

impl Plugin for AssetCachePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetCache::new("assets"))
            .add_event::<AssetLoadedEvent>()
            .add_systems(Update, AssetCache::handle_save_asset_tasks)
            // embedded asset
            .add_systems(Update, handle_embedded_asset_event);
    }
}
