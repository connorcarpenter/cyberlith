use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource};
use naia_bevy_server::UserKey;

use vortex_proto::types::TabId;

struct UserTabState {
    current_tab: Option<TabId>,
    tabs: HashMap<TabId, Entity>,
}

impl Default for UserTabState {
    fn default() -> Self {
        Self {
            current_tab: None,
            tabs: HashMap::new(),
        }
    }
}

#[derive(Resource)]
pub struct TabManager {
    users: HashMap<UserKey, UserTabState>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            users: HashMap::new()
        }
    }
}

impl TabManager {
    pub fn open_tab(&mut self, user_key: &UserKey, tab_id: &TabId, file_entity: &Entity) {
        if !self.users.contains_key(user_key) {
            self.users.insert(user_key.clone(), UserTabState::default());
        }

        let user_state = self.users.get_mut(user_key).unwrap();
        user_state.tabs.insert(tab_id.clone(), file_entity.clone());
    }

    pub fn close_tab(&mut self, user_key: &UserKey, tab_id: &TabId) {
        let mut remove = false;
        if let Some(user_state) = self.users.get_mut(user_key) {
            if user_state.current_tab == Some(tab_id.clone()) {
                user_state.current_tab = None;
            }

            user_state.tabs.remove(tab_id);
            if user_state.tabs.is_empty() {
                remove = true;
            }
        }
        if remove {
            self.users.remove(user_key);
        }
    }

    pub fn select_tab(&mut self, user_key: &UserKey, tab_id: &TabId) {
        if let Some(user_state) = self.users.get_mut(user_key) {
            if user_state.tabs.contains_key(tab_id) {
                user_state.current_tab = Some(tab_id.clone());
            } else {
                panic!("User does not have tab {}", tab_id);
            }
        }
    }
}