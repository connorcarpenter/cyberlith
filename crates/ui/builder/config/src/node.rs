use crate::{
    panel::Panel, style::StyleId, widget::Widget, widget::WidgetKind, Button, Spinner, Text,
    Textbox,
};

#[derive(Clone)]
pub struct UiNode {
    style_id: Option<StyleId>,
    pub init_visible: bool,
    pub widget: Widget,
}

impl UiNode {
    pub(crate) fn new(widget: Widget) -> Self {
        Self {
            style_id: None,
            init_visible: true,
            widget,
        }
    }

    pub fn style_id(&self) -> Option<StyleId> {
        self.style_id
    }

    pub fn set_style_id(&mut self, style_id: StyleId) {
        if self.style_id.is_some() {
            panic!("Node already has a style_id");
        }
        self.style_id = Some(style_id);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.init_visible = visible;
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

    pub fn widget_spinner_ref(&self) -> Option<&Spinner> {
        match &self.widget {
            Widget::Spinner(spinner) => Some(spinner),
            _ => None,
        }
    }

    pub fn widget_spinner_mut(&mut self) -> Option<&mut Spinner> {
        match &mut self.widget {
            Widget::Spinner(spinner) => Some(spinner),
            _ => None,
        }
    }
}
