
use crate::{Panel, Text};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WidgetKind {
    Panel,
    Text,
}

#[derive(Clone)]
pub enum Widget {
    Panel(Panel),
    Text(Text)
}

impl Widget {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Widget::Panel(_) => WidgetKind::Panel,
            Widget::Text(_) => WidgetKind::Text
        }
    }
}