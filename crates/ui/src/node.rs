use crate::{panel::Panel, style::StyleId, widget::Widget, widget::WidgetKind, Button, Text, Textbox};

#[derive(Clone)]
pub struct UiNode {
    pub style_ids: Vec<StyleId>,
    pub widget: Widget,
}

impl UiNode {
    pub(crate) fn new(widget: Widget) -> Self {
        Self {
            style_ids: Vec::new(),
            widget,
        }
    }

    pub fn widget_kind(&self) -> WidgetKind {
        self.widget.kind()
    }

    pub fn widget_panel_ref(&self) -> Option<&Panel> {
        match &self.widget {
            Widget::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_panel_mut(&mut self) -> Option<&mut Panel> {
        match &mut self.widget {
            Widget::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_text_ref(&self) -> Option<&Text> {
        match &self.widget {
            Widget::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_text_mut(&mut self) -> Option<&mut Text> {
        match &mut self.widget {
            Widget::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_button_ref(&self) -> Option<&Button> {
        match &self.widget {
            Widget::Button(button) => Some(button),
            _ => None,
        }
    }

    pub fn widget_button_mut(&mut self) -> Option<&mut Button> {
        match &mut self.widget {
            Widget::Button(button) => Some(button),
            _ => None,
        }
    }

    pub fn widget_textbox_ref(&self) -> Option<&Textbox> {
        match &self.widget {
            Widget::Textbox(textbox) => Some(textbox),
            _ => None,
        }
    }

    pub fn widget_textbox_mut(&mut self) -> Option<&mut Textbox> {
        match &mut self.widget {
            Widget::Textbox(textbox) => Some(textbox),
            _ => None,
        }
    }
}
