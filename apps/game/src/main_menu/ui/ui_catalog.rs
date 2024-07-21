use std::collections::HashMap;

use bevy_ecs::system::Resource;

use game_engine::{asset::AssetId, ui::UiHandle};

use crate::main_menu::ui::UiKey;

#[derive(Resource)]
pub struct UiCatalog {
    // key, (handle, loaded)
    uis: HashMap<UiKey, (UiHandle, bool)>,
    ui_handle_to_key: HashMap<UiHandle, UiKey>,
}

impl Default for UiCatalog {
    fn default() -> Self {
        let mut me = Self {
            uis: HashMap::new(),
            ui_handle_to_key: HashMap::new(),
        };

        me.insert_ui(
            UiKey::MainMenu,
            UiHandle::new(AssetId::from_str("kmqkp9").unwrap()),
        );
        me.insert_ui(
            UiKey::HostMatch,
            UiHandle::new(AssetId::from_str("htytzu").unwrap()),
        );
        me.insert_ui(
            UiKey::JoinMatch,
            UiHandle::new(AssetId::from_str("qqxe6s").unwrap()),
        );
        me.insert_ui(
            UiKey::JoinMatchLobbyItem,
            UiHandle::new(AssetId::from_str("pup52m").unwrap()),
        );
        me.insert_ui(
            UiKey::MessageList,
            UiHandle::new(AssetId::from_str("ngffab").unwrap()),
        );
        me.insert_ui(
            UiKey::MessageListDayDivider,
            UiHandle::new(AssetId::from_str("3wnz6n").unwrap()),
        );
        me.insert_ui(
            UiKey::MessageListUsernameAndMessage,
            UiHandle::new(AssetId::from_str("ddbxab").unwrap()),
        );
        me.insert_ui(
            UiKey::MessageListMessage,
            UiHandle::new(AssetId::from_str("cxc6zk").unwrap()),
        );
        me.insert_ui(
            UiKey::UserListItem,
            UiHandle::new(AssetId::from_str("8ywqfp").unwrap()),
        );

        me
    }
}

impl UiCatalog {
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
