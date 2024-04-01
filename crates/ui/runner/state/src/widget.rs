use ui_runner_config::{WidgetKind, WidgetR};

use crate::{panel::PanelState, button::ButtonState, text::TextState, textbox::TextboxState};

#[derive(Clone)]
pub enum WidgetState {
    Panel(PanelState),
    Text(TextState),
    Button(ButtonState),
    Textbox(TextboxState),
}

impl WidgetState {
    pub(crate) fn from_widget(widget: &WidgetR) -> Self {
        match widget.kind() {
            WidgetKind::Panel => Self::Panel(PanelState::new()),
            WidgetKind::Text => Self::Text(TextState::new()),
            WidgetKind::Button => Self::Button(ButtonState::new()),
            WidgetKind::Textbox => Self::Textbox(TextboxState::new()),
        }
    }
}