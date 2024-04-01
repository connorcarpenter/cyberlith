
use ui_types::{Navigation, Textbox};

#[derive(Clone)]
pub struct TextboxR {
    pub id_str: String,
    pub navigation: Navigation,
}

impl From<Textbox> for TextboxR {
    fn from(value: Textbox) -> Self {
        Self {
            id_str: value.id_str,
            navigation: value.navigation,
        }
    }
}

impl TextboxR {
    pub fn new(id_str: &str) -> Self {
        Self {
            id_str: id_str.to_string(),
            navigation: Navigation::new(),
        }
    }
}