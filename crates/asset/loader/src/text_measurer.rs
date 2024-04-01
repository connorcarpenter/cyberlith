use ui_runner_config::TextMeasurer;

use crate::IconData;

pub struct UiTextMeasurer<'a> {
    icon_data: &'a IconData,
}

impl<'a> UiTextMeasurer<'a> {
    pub fn new(icon_data: &'a IconData) -> Self {
        Self { icon_data }
    }
}

impl<'a> TextMeasurer for UiTextMeasurer<'a> {
    fn get_raw_char_width(&self, subimage: usize) -> f32 {
        if subimage == 0 {
            return 40.0;
        }
        self.icon_data.get_frame_width(subimage).unwrap_or(0.0)
    }

    fn get_raw_char_height(&self, _subimage: usize) -> f32 {
        200.0
    }
}
