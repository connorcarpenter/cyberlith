use ui_runner_config::WidgetKind;

use crate::{
    button::ButtonStyleState, panel::PanelStyleState, spinner::SpinnerStyleState,
    text::TextStyleState, textbox::TextboxStyleState,
};

#[derive(Clone)]
pub enum StyleState {
    Panel(PanelStyleState),
    Text(TextStyleState),
    Button(ButtonStyleState),
    Textbox(TextboxStyleState),
    Spinner(SpinnerStyleState),
    UiContainer,
}

impl StyleState {
    pub fn from_kind(kind: &WidgetKind) -> Self {
        match kind {
            WidgetKind::Panel => Self::Panel(PanelStyleState::new()),
            WidgetKind::Text => Self::Text(TextStyleState::new()),
            WidgetKind::Button => Self::Button(ButtonStyleState::new()),
            WidgetKind::Textbox => Self::Textbox(TextboxStyleState::new()),
            WidgetKind::Spinner => Self::Spinner(SpinnerStyleState::new()),
            WidgetKind::UiContainer => Self::UiContainer,
        }
    }

    pub fn widget_kind(&self) -> WidgetKind {
        match self {
            Self::Panel(_) => WidgetKind::Panel,
            Self::Text(_) => WidgetKind::Text,
            Self::Button(_) => WidgetKind::Button,
            Self::Textbox(_) => WidgetKind::Textbox,
            Self::Spinner(_) => WidgetKind::Spinner,
            Self::UiContainer => WidgetKind::UiContainer,
        }
    }
}
