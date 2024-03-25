pub trait TextMeasurer {
    fn get_raw_char_width(&self, subimage: usize) -> f32;
    fn get_raw_char_height(&self, subimage: usize) -> f32;
}