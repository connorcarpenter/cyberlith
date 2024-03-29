use render_api::base::Color;
use ui_layout::TextMeasurer;

#[derive(Clone)]
pub struct Text {
    pub text: String,
}

impl Text {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }

    pub fn inner_text(&self) -> &str {
        &self.text
    }

    pub fn get_subimage_indices(text: &str) -> Vec<usize> {
        let mut output = Vec::new();

        for c in text.chars() {

            let c: u8 = if c.is_ascii() {
                c as u8
            } else {
                42 // asterisk
            };
            let subimage_index = (c - 32) as usize;
            output.push(subimage_index);
        }

        output
    }

    pub fn get_raw_text_rects(
        text_measurer: &dyn TextMeasurer,
        subimage_indices: &[usize],
    ) -> (Vec<f32>, f32) {
        let mut widths = Vec::new();

        let mut width = 0.0;
        widths.push(width);

        for subimage_index in subimage_indices {
            if width > 0.0 {
                width += 8.0; // between character spacing - TODO: replace with config
            }

            // get character width in order to move cursor appropriately
            let icon_width = text_measurer.get_raw_char_width(*subimage_index);

            width += icon_width;

            widths.push(width);
        }

        (widths, text_measurer.get_raw_char_height(0))
    }

    pub fn measure_raw_text_size(
        text_measurer: &dyn TextMeasurer,
        text: &str,
    ) -> (f32, f32) {
        let subimage_indices = Self::get_subimage_indices(text);
        let (widths, height) = Self::get_raw_text_rects(text_measurer, &subimage_indices);
        (if let Some(width) = widths.last() {
            *width
        } else {
            0.0
        }, height)
    }
}

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub background_color: Option<Color>,
    pub background_alpha: Option<f32>,
}

impl TextStyle {
    pub fn empty() -> Self {
        Self {
            background_color: None,
            background_alpha: None,
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
}