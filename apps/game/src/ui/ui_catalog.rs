use std::collections::HashMap;

use bevy_ecs::system::Resource;

use game_engine::{ui::UiHandle, asset::AssetId};

use crate::ui::UiKey;

#[derive(Resource)]
pub struct UiCatalog {
    // key, (handle, loaded)
    uis: HashMap<UiKey, (UiHandle, bool)>,
    ui_handle_to_key: HashMap<UiHandle, UiKey>,
}

impl UiCatalog {
    pub fn game_main_menu_ui() -> AssetId { AssetId::from_str("kmqkp9").unwrap() }

    pub fn game_host_match_ui() -> AssetId { AssetId::from_str("htytzu").unwrap() }

    pub fn game_global_chat_ui() -> AssetId { AssetId::from_str("ngffab").unwrap() }

    pub fn new() -> Self {
        let mut me = Self {
            uis: HashMap::new(),
            ui_handle_to_key: HashMap::new(),
        };

        me.insert_ui(UiKey::MainMenu, UiHandle::new(Self::game_main_menu_ui()));
        me.insert_ui(UiKey::HostMatch, UiHandle::new(Self::game_host_match_ui()));
        me.insert_ui(UiKey::GlobalChat, UiHandle::new(Self::game_global_chat_ui()));

        me
    }

    fn insert_ui(&mut self, key: UiKey, handle: UiHandle) {
        self.uis.insert(key, (handle, false));
        self.ui_handle_to_key.insert(handle, key);
    }

    pub fn get_is_loaded(&self, key: UiKey) -> bool {
        self.uis.get(&key).unwrap().1
    }

    pub fn set_loaded(&mut self, key: UiKey) {
        let entry = self.uis.get_mut(&key).unwrap();
        entry.1 = true;
    }

    pub fn get_ui_handle(&self, key: UiKey) -> UiHandle {
        self.uis.get(&key).unwrap().0
    }

    pub fn has_ui_key(&self, handle: &UiHandle) -> bool {
        self.ui_handle_to_key.contains_key(handle)
    }

    pub fn get_ui_key(&self, handle: &UiHandle) -> UiKey {
        *self.ui_handle_to_key.get(handle).unwrap()
    }
}
