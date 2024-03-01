use bevy_ecs::{event::{EventWriter, EventReader}, change_detection::ResMut};

use naia_serde::{BitReader, Serde};

use asset_render::{AssetManager, AssetMetadataSerde, AssetMetadataStore, EmbeddedAssetEvent};
use filesystem::FileSystemManager;
use session_server_naia_proto::messages::LoadAssetWithData;

use crate::asset_cache::{AssetCache, AssetLoadedEvent};

pub fn handle_embedded_asset_event(
    mut asset_cache: ResMut<AssetCache>,
    mut asset_manager: ResMut<AssetManager>,
    mut file_system_manager: ResMut<FileSystemManager>,
    mut metadata_store: ResMut<AssetMetadataStore>,
    mut asset_loaded_event_writer: EventWriter<AssetLoadedEvent>,
    mut embedded_asset_event_reader: EventReader<EmbeddedAssetEvent>,
) {
    for event in embedded_asset_event_reader.read() {
        let asset_id = event.asset_id;
        let metadata_bytes = &event.metadata;
        let data_bytes = &event.data;

        let mut bit_reader = BitReader::new(metadata_bytes);
        let metadata = AssetMetadataSerde::de(&mut bit_reader).unwrap();

        let message = LoadAssetWithData::new(asset_id, metadata.asset_type, metadata.etag, data_bytes.to_vec());

        asset_cache.handle_load_asset_with_data_message(
            &mut asset_manager,
            &mut asset_loaded_event_writer,
            &mut file_system_manager,
            &mut metadata_store,
            message
        );
    }
}