use std::slice::Iter;

use ui_layout::NodeId;

use crate::{Button, Panel, Spinner, Text, Textbox, UiContainer};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum WidgetKind {
    Panel,
    Text,
    Button,
    Textbox,
    Spinner,
    UiContainer,
}

impl WidgetKind {
    pub fn has_children(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => true,
            WidgetKind::Textbox => false,
            WidgetKind::Spinner => false,
            WidgetKind::UiContainer => false,
        }
    }

    pub fn can_solid(&self) -> bool {
        match self {
            WidgetKind::Panel => true,
            WidgetKind::Text => false,
            WidgetKind::Button => false,
            WidgetKind::Textbox => false,
            WidgetKind::Spinner => false,
            WidgetKind::UiContainer => false,
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            WidgetKind::Panel => false,
            WidgetKind::Text => true,
            WidgetKind::Button => false,
            WidgetKind::Textbox => false,
            WidgetKind::Spinner => false,
            WidgetKind::UiContainer => false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Widget {
    Panel(Panel),
    Text(Text),
    Button(Button),
    Textbox(Textbox),
    Spinner(Spinner),
    UiContainer(UiContainer),
}

impl Widget {
    pub fn kind(&self) -> WidgetKind {
        match self {
            Self::Panel(_) => WidgetKind::Panel,
            Self::Text(_) => WidgetKind::Text,
            Self::Button(_) => WidgetKind::Button,
            Self::Textbox(_) => WidgetKind::Textbox,
            Self::Spinner(_) => WidgetKind::Spinner,
            Self::UiContainer(_) => WidgetKind::UiContainer,
        }
    }

    pub fn children(&self) -> Option<Iter<NodeId>> {
        match self {
            Self::Panel(panel) => Some(panel.children.iter()),
            Self::Button(button) => Some(button.panel.children.iter()),
            _ => None,
        }
    }

    pub fn clear_children(&mut self) {
        match self {
            Self::Panel(panel) => panel.children.clear(),
            Self::Button(button) => button.panel.children.clear(),
            _ => (),
        }
    }
}
