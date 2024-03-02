use bevy_app::{App, Plugin, Update};

use crate::UiManager;

// Plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // AssetManager
            .init_resource::<UiManager>()
            .add_systems(Update, UiManager::update)
        ;
    }
}
