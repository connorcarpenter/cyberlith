use bevy_app::{App, Plugin, PostUpdate, PreUpdate, Update};

use clipboard::ClipboardPlugin;

use crate::UiManager;

// Plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ClipboardPlugin>() {
            app.add_plugins(ClipboardPlugin);
        }

        app
            // UiManager
            .init_resource::<UiManager>()
            .add_systems(Update, UiManager::sync)
            .add_systems(PreUpdate, UiManager::prepare_cursor_change)
            .add_systems(Update, UiManager::process_ui_global_events)
            .add_systems(Update, UiManager::process_ui_node_events)
            .add_systems(PostUpdate, UiManager::process_cursor_change)
            .add_systems(Update, UiManager::update_blinkiness);
    }
}
