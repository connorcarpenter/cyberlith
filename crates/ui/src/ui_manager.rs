use bevy_ecs::system::Resource;

use asset_id::AssetId;
use asset_render::{AssetHandle, IconData};

#[derive(Resource)]
pub struct UiManager {
    font_handle_opt: Option<AssetHandle<IconData>>,
}

impl Default for UiManager {
    fn default() -> Self {
        Self {
            font_handle_opt: None,
        }
    }
}

impl UiManager {

    // called as a system
    pub fn update() {

    }

    pub fn set_font(&mut self, icon_asset_id: &AssetId) {
        self.font_handle_opt = Some(AssetHandle::new(icon_asset_id.clone()));
    }
}