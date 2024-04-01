use ui_runner_config::WidgetKind;

use crate::textbox::TextboxState;

#[derive(Clone)]
pub enum WidgetState {
    Textbox(TextboxState),
    // more here later perhaps
    None,
}

impl WidgetState {
    pub(crate) fn from_widget(widget: &WidgetKind) -> Self {
        match widget {
            WidgetKind::Textbox => Self::Textbox(TextboxState::new()),
            _ => Self::None,
        }
    }
}
