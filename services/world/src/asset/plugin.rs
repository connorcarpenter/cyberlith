use bevy_app::{App, Plugin, Update};

use crate::asset::{asset_manager, AssetManager};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AssetManager>()
            .add_systems(Update, asset_manager::update);
    }
}