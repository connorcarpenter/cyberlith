use ui_builder_config::{Widget, WidgetKind};

use crate::{textbox::TextboxR, text::TextR, button::ButtonR, panel::PanelR};

#[derive(Clone)]
pub enum WidgetR {
    Panel(PanelR),
    Text(TextR),
    Button(ButtonR),
    Textbox(TextboxR),
}

impl From<Widget> for WidgetR {
    fn from(value: Widget) -> Self {
        match value {
            Widget::Panel(w) => WidgetR::Panel(w.into()),
            Widget::Text(w) => WidgetR::Text(w.into()),
            Widget::Button(w) => WidgetR::Button(w.into()),
            Widget::Textbox(w) => WidgetR::Textbox(w.into()),
        }
    }
}

impl WidgetR {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Self::Panel(_) => WidgetKind::Panel,
            Self::Text(_) => WidgetKind::Text,
            Self::Button(_) => WidgetKind::Button,
            Self::Textbox(_) => WidgetKind::Textbox,
        }
    }
}