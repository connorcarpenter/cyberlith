use render_api::base::Color;

#[derive(Clone, Debug)]
pub struct Text {
    pub init_text: String,
}

impl Text {
    pub fn new(init_text: &str) -> Self {
        Self {
            init_text: init_text.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>,
    pub text_color: Option<Color>,
}

impl TextStyle {
    pub fn merge(&mut self, other: &Self) {
        self.background_color = other.background_color.or(self.background_color);
        self.background_alpha = other.background_alpha.or(self.background_alpha);
        self.text_color = other.text_color.or(self.text_color);
    }
}

impl TextStyle {
    pub fn empty() -> Self {
        Self {
            background_color: None,
            background_alpha: None,
            text_color: None,
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
