use ui_runner_config::Widget;

use crate::{text::TextState, textbox::TextboxState, UiContainerState};

#[derive(Clone)]
pub enum WidgetState {
    Text(TextState),
    Textbox(TextboxState),
    UiContainer(UiContainerState),
    // more here later perhaps
    None,
}

impl WidgetState {
    pub(crate) fn from_widget(widget: &Widget) -> Self {
        match widget {
            Widget::Textbox(textbox) => Self::Textbox(TextboxState::new(textbox)),
            Widget::Text(text) => Self::Text(TextState::new(text)),
            Widget::UiContainer(_) => Self::UiContainer(UiContainerState::new()),
            _ => Self::None,
        }
    }
}
