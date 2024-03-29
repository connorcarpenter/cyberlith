use ui_types::{UiNode, WidgetKind};

use crate::textbox_input_state::TextboxInputState;

#[derive(Clone)]
pub struct UiNodeInputState {
    pub textbox_state_opt: Option<TextboxInputState>,
}

impl UiNodeInputState {
    pub(crate) fn new(textbox_state_opt: Option<TextboxInputState>) -> Self {
        Self {
            textbox_state_opt,
        }
    }

    pub(crate) fn from_node(ui_node: &UiNode) -> Self {
        match ui_node.widget_kind() {
            WidgetKind::Textbox => {
                let textbox_state = TextboxInputState::new();
                Self::new(Some(textbox_state))
            }
            _ => Self::new(None),
        }
    }

    pub fn widget_textbox_ref(&self) -> Option<&TextboxInputState> {
        self.textbox_state_opt.as_ref()
    }

    pub fn widget_textbox_mut(&mut self) -> Option<&mut TextboxInputState> {
        self.textbox_state_opt.as_mut()
    }
}
