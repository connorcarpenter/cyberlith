use std::collections::HashMap;

use bevy_ecs::system::Resource;

use game_engine::{ui::UiHandle, asset::AssetId};

use crate::ui::UiKey;

#[derive(Resource)]
pub struct UiCatalog {
    ui_key_to_handle: HashMap<UiKey, UiHandle>,
    ui_handle_to_key: HashMap<UiHandle, UiKey>,
}

impl UiCatalog {
    pub fn game_main_menu_ui() -> AssetId { AssetId::from_str("kmqkp9").unwrap() }

    pub fn game_host_match_ui() -> AssetId { AssetId::from_str("htytzu").unwrap() }

    pub fn game_global_chat_ui() -> AssetId { AssetId::from_str("ngffab").unwrap() }

    pub fn new() -> Self {
        Self {
            ui_key_to_handle: HashMap::new(),
            ui_handle_to_key: HashMap::new(),
        }
    }

    pub fn insert_ui(&mut self, key: UiKey, handle: UiHandle) {
        self.ui_key_to_handle.insert(key, handle);
        self.ui_handle_to_key.insert(handle, key);
    }

    pub fn get_ui_handle(&self, key: UiKey) -> &UiHandle {
        self.ui_key_to_handle.get(&key).unwrap()
    }

    pub fn get_ui_key(&self, handle: &UiHandle) -> &UiKey {
        self.ui_handle_to_key.get(handle).unwrap()
    }
}
