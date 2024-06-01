use render_api::base::Color;

#[derive(Clone, Debug)]
pub struct Spinner {
    pub id_str: String,
}

impl Spinner {
    pub fn new(id_str: &str) -> Self {
        Self {
            id_str: id_str.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpinnerStyle {
    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>,
    pub spinner_color: Option<Color>,
}

impl SpinnerStyle {
    pub fn merge(&mut self, other: &Self) {
        self.background_color = other.background_color.or(self.background_color);
        self.background_alpha = other.background_alpha.or(self.background_alpha);
        self.spinner_color = other.spinner_color.or(self.spinner_color);
    }
}

impl SpinnerStyle {
    pub fn empty() -> Self {
        Self {
            background_color: None,
            background_alpha: None,
            spinner_color: None,
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

    pub fn spinner_color(&self) -> Option<Color> {
        self.spinner_color
    }

    pub fn set_spinner_color(&mut self, val: Color) {
        self.spinner_color = Some(val);
    }
}
