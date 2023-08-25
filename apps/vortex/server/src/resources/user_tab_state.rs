use std::collections::HashMap;

use vortex_proto::{resources::FileEntryKey, types::TabId};

pub struct UserTabState {
    current_tab: Option<TabId>,
    tabs: HashMap<TabId, FileEntryKey>,
}

impl Default for UserTabState {
    fn default() -> Self {
        Self {
            current_tab: None,
            tabs: HashMap::new(),
        }
    }
}

impl UserTabState {
    pub fn has_tabs(&self) -> bool {
        !self.tabs.is_empty()
    }

    pub fn remove_tab(&mut self, tab_id: &TabId) -> Option<FileEntryKey> {
        if let Some(file_key) = self.tabs.remove(tab_id) {
            Some(file_key)
        } else {
            None
        }
    }

    pub fn insert_tab(&mut self, tab_id: TabId, file_key: FileEntryKey) {
        self.tabs.insert(tab_id, file_key);
    }

    pub fn has_tab_id(&self, tab_id: &TabId) -> bool {
        self.tabs.contains_key(tab_id)
    }

    pub fn set_current_tab(&mut self, tab_id_opt: Option<TabId>) {
        self.current_tab = tab_id_opt;
    }

    pub fn current_tab(&self) -> Option<TabId> {
        self.current_tab
    }

    pub fn tab_file_key(&self, tab_id: &TabId) -> Option<FileEntryKey> {
        if let Some(file_key) = self.tabs.get(tab_id) {
            Some(file_key.clone())
        } else {
            None
        }
    }
}
