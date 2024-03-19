use crate::{widget::WidgetKind, panel::Panel, style::StyleId, widget::Widget, Text, Button};

#[derive(Clone)]
pub struct UiNode {
    pub visible: bool,
    pub style_ids: Vec<StyleId>,
    pub widget: Widget,
}

impl UiNode {
    pub(crate) fn new(widget: Widget) -> Self {
        Self {
            visible: true,
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
}
