
use render_api::base::Color;

use crate::{Panel, PanelStyle, Navigation};

#[derive(Clone)]
pub struct Textbox {
    pub panel: Panel,
    pub id_str: String,
    pub navigation: Navigation,
}

impl Textbox {
    pub fn new(id_str: &str) -> Self {
        Self {
            panel: Panel::new(),
            id_str: id_str.to_string(),
            navigation: Navigation::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextboxStyle {
    pub panel: PanelStyle,

    pub hover_color: Option<Color>,
    pub active_color: Option<Color>,
    pub select_color: Option<Color>,
}

impl TextboxStyle {
    pub fn empty() -> Self {
        Self {
            panel: PanelStyle::empty(),

            hover_color: None,
            active_color: None,
            select_color: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.panel.background_alpha()
    }

    pub fn set_background_alpha(&mut self, val: f32) {
        self.panel.set_background_alpha(val);
    }

    pub fn hover_color(&self) -> Option<Color> {
        self.hover_color
    }

    pub fn set_hover_color(&mut self, val: Color) {
        self.hover_color = Some(val);
    }

    pub fn active_color(&self) -> Option<Color> {
        self.active_color
    }

    pub fn set_active_color(&mut self, val: Color) {
        self.active_color = Some(val);
    }

    pub fn selection_color(&self) -> Option<Color> {
        self.select_color
    }

    pub fn set_selection_color(&mut self, val: Color) {
        self.select_color = Some(val);
    }
}