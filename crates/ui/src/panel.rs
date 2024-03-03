use crate::{Style, UiId};

#[derive(Default, Clone)]
pub(crate) struct Panel {
    pub(crate) children: Vec<UiId>,
    pub(crate) visible: bool,
    pub(crate) style: Style,
}

impl Panel {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}