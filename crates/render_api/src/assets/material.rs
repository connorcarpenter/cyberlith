use std::default::Default;

use crate::Color;

#[derive(Default)]
pub struct Material {}

impl From<Color> for Material {
    fn from(value: Color) -> Self {
        Self {}
    }
}
