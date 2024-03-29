use crate::{Button, textbox::TextboxState, text::TextState, Panel, panel::PanelState, Text, Textbox, button::ButtonState};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WidgetKind {
    Panel,
    Text,
    Button,
    Textbox,
}

impl WidgetKind {
    pub fn has_children(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => true,
            WidgetKind::Textbox => false,
        }
    }

    pub fn can_solid(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => false,
            WidgetKind::Textbox => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            WidgetKind::Panel => false,
            WidgetKind::Text => true,
            WidgetKind::Button => false,
            WidgetKind::Textbox => false,
        }
    }
}

#[derive(Clone)]
pub enum Widget {
    Panel(Panel),
    Text(Text),
    Button(Button),
    Textbox(Textbox),
}

impl Widget {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Self::Panel(_) => WidgetKind::Panel,
            Self::Text(_) => WidgetKind::Text,
            Self::Button(_) => WidgetKind::Button,
            Self::Textbox(_) => WidgetKind::Textbox,
        }
    }
}

#[derive(Clone)]
pub enum WidgetState {
    Panel(PanelState),
    Text(TextState),
    Button(ButtonState),
    Textbox(TextboxState),
}

impl WidgetState {
    pub(crate) fn from_widget(widget: &Widget) -> Self {
        match widget.kind() {
            WidgetKind::Panel => Self::Panel(PanelState::new()),
            WidgetKind::Text => Self::Text(TextState::new()),
            WidgetKind::Button => Self::Button(ButtonState::new()),
            WidgetKind::Textbox => Self::Textbox(TextboxState::new()),
        }
    }
}