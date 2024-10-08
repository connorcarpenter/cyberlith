use crate::{
    panel::Panel, style::StyleId, widget::Widget, widget::WidgetKind, Button, Spinner, Text,
    Textbox,
};

#[derive(Clone, Debug)]
pub struct UiNode {
    style_id: Option<StyleId>,
    pub init_visible: bool,
    pub widget: Widget,
    pub id_str: Option<String>,
}

impl UiNode {
    pub fn new(id_str_opt: Option<&str>, widget: Widget) -> Self {
        Self {
            style_id: None,
            init_visible: true,
            widget,
            id_str: id_str_opt.map(|s| s.to_string()),
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

    pub fn clear_style_id(&mut self) {
        self.style_id = None;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.init_visible = visible;
    }

    pub fn id_str_opt(&self) -> Option<String> {
        self.id_str.clone()
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
