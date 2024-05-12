use ui_runner_config::Widget;

use crate::textbox::TextboxState;

#[derive(Clone)]
pub enum WidgetState {
    Textbox(TextboxState),
    // more here later perhaps
    None,
}

impl WidgetState {
    pub(crate) fn from_widget(widget: &Widget) -> Self {
        match widget {
            Widget::Textbox(textbox) => Self::Textbox(TextboxState::new(textbox)),
            _ => Self::None,
        }
    }
}