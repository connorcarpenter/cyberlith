use bevy_app::{App, Plugin, PostUpdate, PreUpdate, Startup, Update};

use crate::{AssetManager, AssetMetadataStore, EmbeddedAssetEvent};

// Plugin
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            // AssetManager
            .init_resource::<AssetManager>()
            .add_systems(Update, AssetManager::sync)
            .add_systems(PreUpdate, AssetManager::prepare_cursor_change)
            .add_systems(Update, AssetManager::process_ui_events)
            .add_systems(PostUpdate, AssetManager::process_cursor_change)
            // AssetMetadataStore
            // TODO: AssetMetadataStore "assets" path here should be a config param somehow
            .insert_resource(AssetMetadataStore::new("assets"))
            .add_systems(Startup, AssetMetadataStore::startup)
            .add_systems(Update, AssetMetadataStore::handle_metadata_tasks)
            // Embedded stuff
            .add_event::<EmbeddedAssetEvent>();
    }
}
