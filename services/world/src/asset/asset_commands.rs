use bevy_ecs::system::EntityCommands;

use naia_bevy_server::Server;

use asset_id::AssetId;

use crate::asset::AssetManager;

// AssetCommandsExt
pub trait AssetCommandsExt {
    fn insert_asset<M: Send + Sync + 'static>(
        &mut self,
        asset_manager: &mut AssetManager,
        server: &mut Server,
        asset_id: AssetId,
    ) -> &mut Self;
}

impl AssetCommandsExt for EntityCommands<'_> {
    fn insert_asset<M: Send + Sync + 'static>(
        &mut self,
        asset_manager: &mut AssetManager,
        server: &mut Server,
        asset_id: AssetId,
    ) -> &mut Self {
        let new_ref = asset_manager.create_asset_ref::<M>(&mut self.commands(), server, asset_id);
        self.insert(new_ref);
        self
    }
}
