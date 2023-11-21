
use crate::base::Color;

#[derive(Debug, Clone)]
pub struct CpuMaterial {

    pub name: String,
    pub diffuse: Color,
    pub emissive: Color,
    pub shininess: f32,
}

impl Default for CpuMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            diffuse: Color::WHITE,
            emissive: Color::BLACK,
            shininess: 32.0,
        }
    }
}

impl From<Color> for CpuMaterial {
    fn from(color: Color) -> Self {
        Self {
            diffuse: color,
            ..Default::default()
        }
    }
}
