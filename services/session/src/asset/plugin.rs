
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::IntoSystemConfigs;

use naia_bevy_server::ReceiveEvents;

use super::{asset_manager, asset_manager::AssetManager};

pub struct AssetPlugin {

}

impl AssetPlugin {
    pub fn new(

    ) -> Self {
        Self {

        }
    }
}

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AssetManager::new())
            .add_systems(
                Update,
                asset_manager::update.in_set(ReceiveEvents)
            );
    }
}