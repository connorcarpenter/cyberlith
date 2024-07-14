use std::collections::HashMap;

use bevy_ecs::{event::EventWriter, system::Resource};

use game_engine::{asset::AssetId, logging::info, ui::UiManager};

use crate::ui::events::{
    ResyncLobbyGlobalEvent, ResyncMatchLobbiesEvent, ResyncPublicUserInfoEvent,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssetKey {
    FontIcon,
    PasswordEyeIcon,
}

#[derive(Resource)]
pub struct AssetCatalog {
    // key, (handle, loaded)
    assets: HashMap<AssetKey, (AssetId, bool)>,
    asset_id_to_key: HashMap<AssetId, AssetKey>,
}

impl AssetCatalog {
    pub fn new() -> Self {
        let mut me = Self {
            assets: HashMap::new(),
            asset_id_to_key: HashMap::new(),
        };

        me.insert_asset(AssetKey::FontIcon, AssetId::from_str("34mvvk").unwrap());
        me.insert_asset(
            AssetKey::PasswordEyeIcon,
            AssetId::from_str("qbgz5j").unwrap(),
        );

        me
    }

    fn insert_asset(&mut self, key: AssetKey, asset_id: AssetId) {
        self.assets.insert(key, (asset_id, false));
        self.asset_id_to_key.insert(asset_id, key);
    }

    pub fn get_is_loaded(&self, key: AssetKey) -> bool {
        self.assets.get(&key).unwrap().1
    }

    pub fn set_loaded(&mut self, key: AssetKey) {
        let entry = self.assets.get_mut(&key).unwrap();
        entry.1 = true;
    }

    pub fn get_asset_id(&self, key: AssetKey) -> AssetId {
        self.assets.get(&key).unwrap().0
    }

    pub fn has_asset_key(&self, asset_id: &AssetId) -> bool {
        self.asset_id_to_key.contains_key(asset_id)
    }

    pub fn get_asset_key(&self, asset_id: &AssetId) -> AssetKey {
        *self.asset_id_to_key.get(asset_id).unwrap()
    }
}

pub(crate) fn on_asset_load(
    ui_manager: &mut UiManager,
    asset_catalog: &mut AssetCatalog,
    resync_user_public_info_events: &mut EventWriter<ResyncPublicUserInfoEvent>,
    resync_global_chat_events: &mut EventWriter<ResyncLobbyGlobalEvent>,
    resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
    asset_id: AssetId,
) {
    if !asset_catalog.has_asset_key(&asset_id) {
        return;
    }

    info!(
        "received Asset Loaded Icon Event! (asset_id: {:?})",
        asset_id
    );

    let asset_key = asset_catalog.get_asset_key(&asset_id);
    asset_catalog.set_loaded(asset_key);

    match asset_key {
        AssetKey::FontIcon => {
            ui_manager.set_text_icon_handle(asset_id);
            resync_user_public_info_events.send(ResyncPublicUserInfoEvent);
            resync_global_chat_events.send(ResyncLobbyGlobalEvent::new(true));
            resync_match_lobbies_events.send(ResyncMatchLobbiesEvent);
        }
        AssetKey::PasswordEyeIcon => {
            ui_manager.set_eye_icon_handle(asset_id);
        } // _ => {
          //     unimplemented!("asset load not implemented");
          // }
    }
}
