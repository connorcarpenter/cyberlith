use render_api::base::Color;
use validation::ValidationType;

use crate::Navigation;

#[derive(Clone, Debug)]
pub struct Textbox {
    pub navigation: Navigation,
    pub is_password: bool,
    pub validation: Option<ValidationType>,
}

impl Textbox {
    pub fn new() -> Self {
        Self {
            navigation: Navigation::new(),
            is_password: false,
            validation: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextboxStyle {
    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>,
    pub text_color: Option<Color>,

    pub hover_color: Option<Color>,
    pub active_color: Option<Color>,
    pub select_color: Option<Color>,
}

impl TextboxStyle {
    pub fn merge(&mut self, other: &Self) {
        self.background_color = other.background_color.or(self.background_color);
        self.text_color = other.text_color.or(self.text_color);
        self.background_alpha = other.background_alpha.or(self.background_alpha);
        self.hover_color = other.hover_color.or(self.hover_color);
        self.active_color = other.active_color.or(self.active_color);
        self.select_color = other.select_color.or(self.select_color);
    }
}

impl TextboxStyle {
    pub fn empty() -> Self {
        Self {
            background_color: None,
            text_color: None,
            background_alpha: None,
            hover_color: None,
            active_color: None,
            select_color: None,
        }
    }

    pub fn background_alpha(&self) -> Option<f32> {
        self.background_alpha
    }

    pub fn set_background_alpha(&mut self, val: f32) {
        // validate
        if val < 0.0 || val > 1.0 {
            panic!("background_alpha must be between 0.0 and 1.0");
        }
        if (val * 10.0).fract() != 0.0 {
            panic!("background_alpha must be a multiple of 0.1");
        }

        self.background_alpha = Some(val);
    }

    pub fn text_color(&self) -> Option<Color> {
        self.text_color
    }

    pub fn set_text_color(&mut self, val: Color) {
        self.text_color = Some(val);
    }
}
