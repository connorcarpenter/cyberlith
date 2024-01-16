use std::collections::HashMap;

use vortex_proto::{resources::FileKey, types::TabId};

pub struct UserTabState {
    tabs: HashMap<TabId, FileKey>,
}

impl Default for UserTabState {
    fn default() -> Self {
        Self {
            tabs: HashMap::new(),
        }
    }
}

impl UserTabState {
    pub fn insert_tab(&mut self, tab_id: TabId, file_key: FileKey) {
        self.tabs.insert(tab_id, file_key);
    }

    pub fn remove_tab(&mut self, tab_id: &TabId) -> Option<FileKey> {
        if let Some(file_key) = self.tabs.remove(tab_id) {
            Some(file_key)
        } else {
            None
        }
    }
}
