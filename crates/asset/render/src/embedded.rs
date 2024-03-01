
#[macro_export]
macro_rules! embedded_asset_event {
    ($path: expr) => {{

        let data_bytes = include_bytes!($path);
        let metadata_bytes = include_bytes!(concat!($path, ".meta"));

        game_engine::asset::EmbeddedAssetEvent::from_bytes(data_bytes, metadata_bytes)
    }};
}

use bevy_ecs::event::Event;

#[derive(Event)]
pub struct EmbeddedAssetEvent {
    data: Vec<u8>,
    metadata: Vec<u8>,
}

impl EmbeddedAssetEvent {
    pub fn from_bytes(data: &[u8], metadata: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            metadata: metadata.to_vec(),
        }
    }
}