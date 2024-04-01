use ui_builder_config::{StyleId, UiNode, WidgetKind};

use crate::{widget::WidgetR, textbox::TextboxR, text::TextR, panel::PanelR, button::ButtonR};

#[derive(Clone)]
pub struct UiNodeR {
    style_id: Option<StyleId>,
    pub widget: WidgetR,
}

impl From<UiNode> for UiNodeR {
    fn from(node: UiNode) -> Self {
        Self {
            style_id: node.style_id(),
            widget: node.widget.into(),
        }
    }
}

impl UiNodeR {

    pub fn style_id(&self) -> Option<StyleId> {
        self.style_id
    }

    pub fn widget_kind(&self) -> WidgetKind {
        self.widget.kind()
    }

    pub fn widget_panel_ref(&self) -> Option<&PanelR> {
        match &self.widget {
            WidgetR::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_text_ref(&self) -> Option<&TextR> {
        match &self.widget {
            WidgetR::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_button_ref(&self) -> Option<&ButtonR> {
        match &self.widget {
            WidgetR::Button(button) => Some(button),
            _ => None,
        }
    }

    pub fn widget_textbox_ref(&self) -> Option<&TextboxR> {
        match &self.widget {
            WidgetR::Textbox(textbox) => Some(textbox),
            _ => None,
        }
    }
}
