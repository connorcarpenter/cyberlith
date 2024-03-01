
#[macro_export]
macro_rules! embedded_asset_event {
    ($path: expr) => {{

        let path = std::path::Path::new($path);
        let asset_id_str = path.file_stem().unwrap().to_str().unwrap();

        let data_bytes = include_bytes!($path);
        let metadata_bytes = include_bytes!(concat!($path, ".meta"));

        game_engine::asset::EmbeddedAssetEvent::from_bytes(asset_id_str, data_bytes, metadata_bytes)
    }};
}

use bevy_ecs::event::Event;

use asset_id::AssetId;

#[derive(Event)]
pub struct EmbeddedAssetEvent {
    pub asset_id: AssetId,
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

impl EmbeddedAssetEvent {
    pub fn from_bytes(asset_id_str: &str, data: &[u8], metadata: &[u8]) -> Self {
        let asset_id = AssetId::from_str(asset_id_str).unwrap();
        Self {
            asset_id,
            data: data.to_vec(),
            metadata: metadata.to_vec(),
        }
    }
}