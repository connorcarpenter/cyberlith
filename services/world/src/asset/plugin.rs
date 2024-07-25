use bevy_app::{App, Plugin, Update};

use world_server_naia_proto::components::{Alt1, Main};

use crate::asset::{asset_manager, AssetManager};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AssetManager>()
            .add_systems(Update, asset_manager::update)
            .add_systems(Update, asset_manager::handle_asset_ref_added_events::<Main>)
            .add_systems(Update, asset_manager::handle_asset_ref_added_events::<Alt1>)
            .add_systems(Update, asset_manager::handle_asset_ref_removed_events::<Main>)
            .add_systems(Update, asset_manager::handle_asset_ref_removed_events::<Alt1>);
    }
}