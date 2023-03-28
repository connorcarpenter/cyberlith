use std::default::Default;

#[derive(Clone, Default)]
pub struct Image {
    width: u32,
    height: u32,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
