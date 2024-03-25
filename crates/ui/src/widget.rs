use crate::{Button, Panel, Text, Textbox};

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
            Widget::Panel(_) => WidgetKind::Panel,
            Widget::Text(_) => WidgetKind::Text,
            Widget::Button(_) => WidgetKind::Button,
            Widget::Textbox(_) => WidgetKind::Textbox,
        }
    }
}
