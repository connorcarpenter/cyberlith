
use crate::{Button, Panel, Text};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WidgetKind {
    Panel,
    Text,
    Button,
}

#[derive(Clone)]
pub enum Widget {
    Panel(Panel),
    Text(Text),
    Button(Button),
}

impl Widget {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Widget::Panel(_) => WidgetKind::Panel,
            Widget::Text(_) => WidgetKind::Text,
            Widget::Button(_) => WidgetKind::Button,
        }
    }
}