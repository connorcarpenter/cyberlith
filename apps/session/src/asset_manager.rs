use bevy_ecs::prelude::Resource;

use naia_bevy_server::UserKey;

use asset_id::AssetId;

#[derive(Resource)]
pub struct AssetManager {

}

impl AssetManager {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn user_asset_request(&mut self, user_key: UserKey, asset_id: &AssetId, added: bool) {
        todo!()
    }
}