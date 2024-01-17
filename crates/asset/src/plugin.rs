use bevy_app::{App, Plugin, Update};

use crate::AssetManager;

// Plugin
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManager>()
            .add_systems(Update, AssetManager::sync);
    }
}
