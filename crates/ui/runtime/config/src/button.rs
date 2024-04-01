
use ui_builder_config::{Button, Navigation};

use crate::panel::PanelR;

#[derive(Clone)]
pub struct ButtonR {
    pub panel: PanelR,
    pub id_str: String,
    pub navigation: Navigation,
}

impl From<Button> for ButtonR {
    fn from(value: Button) -> Self {
        Self {
            panel: value.panel.into(),
            id_str: value.id_str,
            navigation: value.navigation,
        }
    }
}

impl ButtonR {
    pub fn new(id_str: &str) -> Self {
        Self {
            panel: PanelR::new(),
            id_str: id_str.to_string(),
            navigation: Navigation::new(),
        }
    }
}