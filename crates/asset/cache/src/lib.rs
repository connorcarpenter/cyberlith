mod plugin;
mod asset_cache;
mod embedded_asset;

pub use plugin::AssetCachePlugin;
pub use asset_cache::{AssetCache, AssetLoadedEvent};