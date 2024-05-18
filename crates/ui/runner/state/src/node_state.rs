use ui_runner_config::UiNode;

use crate::{text::TextState, textbox::TextboxState, UiContainerState, widget::WidgetState};

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

    pub fn widget_ui_container_ref(&self) -> Option<&UiContainerState> {
        match &self.widget {
            WidgetState::UiContainer(ui_container) => Some(ui_container),
            _ => None,
        }
    }

    pub fn widget_ui_container_mut(&mut self) -> Option<&mut UiContainerState> {
        match &mut self.widget {
            WidgetState::UiContainer(ui_container) => Some(ui_container),
            _ => None,
        }
    }
}
