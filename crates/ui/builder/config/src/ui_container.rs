
#[derive(Clone)]
pub struct UiContainer {
    pub id_str: String,
}

impl UiContainer {
    pub fn new(id_str: &str) -> Self {
        Self {
            id_str: id_str.to_string(),
        }
    }
}
