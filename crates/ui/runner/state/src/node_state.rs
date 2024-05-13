use ui_runner_config::UiNode;

use crate::{textbox::TextboxState, widget::WidgetState};
use crate::text::TextState;

#[derive(Clone)]
pub struct UiNodeState {
    pub widget: WidgetState,
}

impl UiNodeState {
    pub(crate) fn new(widget: WidgetState) -> Self {
        Self { widget }
    }

    pub(crate) fn from_node(node: &UiNode) -> Self {
        let widget_state = WidgetState::from_widget(&node.widget);

        Self::new(widget_state)
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
