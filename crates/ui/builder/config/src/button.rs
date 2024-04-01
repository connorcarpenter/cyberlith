use render_api::base::Color;
use ui_layout::NodeId;

use crate::{Panel, PanelStyle};

#[derive(Clone)]
pub struct Button {
    pub panel: Panel,
    pub id_str: String,
    pub navigation: Navigation,
}

impl Button {
    pub fn new(id_str: &str) -> Self {
        Self {
            panel: Panel::new(),
            id_str: id_str.to_string(),
            navigation: Navigation::new(),
        }
    }

    pub fn add_child(&mut self, child_id: NodeId) {
        self.panel.add_child(child_id);
    }
}

#[derive(Clone)]
pub struct Navigation {
    pub left_goes_to: Option<String>,
    pub right_goes_to: Option<String>,
    pub up_goes_to: Option<String>,
    pub down_goes_to: Option<String>,
    pub tab_goes_to: Option<String>,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            left_goes_to: None,
            right_goes_to: None,
            up_goes_to: None,
            down_goes_to: None,
            tab_goes_to: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ButtonStyle {
    pub panel: PanelStyle,

    pub hover_color: Option<Color>,
    pub down_color: Option<Color>,
}

impl ButtonStyle {
    pub fn empty() -> Self {
        Self {
            panel: PanelStyle::empty(),
            hover_color: None,
            down_color: None,
        }
    }

    pub fn set_background_alpha(&mut self, val: f32) {
        self.panel.set_background_alpha(val);
    }
}
