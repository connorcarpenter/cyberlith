use bevy_app::{App, Plugin, PostUpdate, PreUpdate, Startup, Update};

use clipboard::ClipboardPlugin;

use crate::{ui_manager::UiManager, AssetManager, AssetMetadataStore, EmbeddedAssetEvent};

// Plugin
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<ClipboardPlugin>() {
            app.add_plugins(ClipboardPlugin);
        }

        app
            // AssetManager
            .init_resource::<AssetManager>()
            .add_systems(Update, AssetManager::sync)
            // UiManager
            .init_resource::<UiManager>()
            .add_systems(PreUpdate, UiManager::prepare_cursor_change)
            .add_systems(Update, UiManager::process_ui_global_events)
            .add_systems(Update, UiManager::process_ui_node_events)
            .add_systems(PostUpdate, UiManager::process_cursor_change)
            .add_systems(Update, UiManager::update_blinkiness)
            // AssetMetadataStore
            // TODO: AssetMetadataStore "assets" path here should be a config param somehow
            .insert_resource(AssetMetadataStore::new("assets"))
            .add_systems(Startup, AssetMetadataStore::startup)
            .add_systems(Update, AssetMetadataStore::handle_metadata_tasks)
            // Embedded stuff
            .add_event::<EmbeddedAssetEvent>();
    }
}
