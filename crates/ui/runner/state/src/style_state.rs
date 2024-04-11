use ui_runner_config::WidgetKind;

use crate::{
    button::ButtonStyleState, panel::PanelStyleState, text::TextStyleState,
    textbox::TextboxStyleState,
};

pub enum StyleState {
    Panel(PanelStyleState),
    Text(TextStyleState),
    Button(ButtonStyleState),
    Textbox(TextboxStyleState),
}

impl StyleState {
    pub fn from_kind(kind: &WidgetKind) -> Self {
        match kind {
            WidgetKind::Panel => Self::Panel(PanelStyleState::new()),
            WidgetKind::Text => Self::Text(TextStyleState::new()),
            WidgetKind::Button => Self::Button(ButtonStyleState::new()),
            WidgetKind::Textbox => Self::Textbox(TextboxStyleState::new()),
        }
    }
}
