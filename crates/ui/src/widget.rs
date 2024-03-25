use crate::{Button, Panel, Text};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WidgetKind {
    Panel,
    Text,
    Button,
}

impl WidgetKind {
    pub fn has_children(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => true,
        }
    }

    pub fn can_solid(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            WidgetKind::Panel => false,
            WidgetKind::Text => true,
            WidgetKind::Button => false,
        }
    }
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
