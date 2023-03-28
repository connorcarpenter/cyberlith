use std::default::Default;

use crate::Color;

#[derive(Default)]
pub struct StandardMaterial {}

impl From<Color> for StandardMaterial {
    fn from(value: Color) -> Self {
        Self {}
    }
}
