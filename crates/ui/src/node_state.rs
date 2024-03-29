use ui_types::UiNode;

use crate::{button::ButtonState, widget::WidgetState, panel::PanelState, text::TextState, textbox::TextboxState};

#[derive(Clone)]
pub struct UiNodeState {
    pub widget: WidgetState,
}

impl UiNodeState {
    pub(crate) fn new(widget: WidgetState) -> Self {
        Self {
            widget,
        }
    }

    pub(crate) fn from_node(ui_node: &UiNode) -> Self {
        let widget_state = WidgetState::from_widget(&ui_node.widget);

        Self::new(widget_state)
    }

    pub fn widget_panel_ref(&self) -> Option<&PanelState> {
        match &self.widget {
            WidgetState::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_panel_mut(&mut self) -> Option<&mut PanelState> {
        match &mut self.widget {
            WidgetState::Panel(panel) => Some(panel),
            _ => None,
        }
    }

    pub fn widget_text_ref(&self) -> Option<&TextState> {
        match &self.widget {
            WidgetState::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_text_mut(&mut self) -> Option<&mut TextState> {
        match &mut self.widget {
            WidgetState::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn widget_button_ref(&self) -> Option<&ButtonState> {
        match &self.widget {
            WidgetState::Button(button) => Some(button),
            _ => None,
        }
    }

    pub fn widget_button_mut(&mut self) -> Option<&mut ButtonState> {
        match &mut self.widget {
            WidgetState::Button(button) => Some(button),
            _ => None,
        }
    }

    pub fn widget_textbox_ref(&self) -> Option<&TextboxState> {
        match &self.widget {
            WidgetState::Textbox(textbox) => Some(textbox),
            _ => None,
        }
    }

    pub fn widget_textbox_mut(&mut self) -> Option<&mut TextboxState> {
        match &mut self.widget {
            WidgetState::Textbox(textbox) => Some(textbox),
            _ => None,
        }
    }
}
