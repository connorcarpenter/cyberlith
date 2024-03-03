use std::collections::HashMap;

use crate::{UiId};
use crate::style::Style;

pub struct PanelStore {
    map: HashMap<UiId, Panel>
}

impl PanelStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    pub fn insert(&mut self, uiid: UiId, panel: Panel) {
        self.map.insert(uiid, panel);
    }

    pub fn get(&self, uiid: &UiId) -> Option<&Panel> {
        self.map.get(uiid)
    }

    pub fn get_mut(&mut self, uiid: &UiId) -> Option<&mut Panel> {
        self.map.get_mut(uiid)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&UiId, &Panel)> {
        self.map.iter()
    }
}

#[derive(Default, Clone)]
pub struct Panel {
    pub(crate) children: Vec<UiId>,
    pub(crate) visible: bool,
    pub(crate) style: Style,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            visible: true,
            ..Default::default()
        }
    }
}