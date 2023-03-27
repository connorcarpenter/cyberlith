use std::default::Default;

#[derive(Clone, Default)]
pub struct Image {
    width: f32,
    height: f32,
}

impl Image {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
