use std::default::Default;

pub struct Color {}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {}
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Self {}
    }
}

#[derive(Default)]
pub enum ClearColorConfig {
    #[default]
    Default,
    Custom(Color),
    None,
}
