use crate::base::Color;

#[derive(Debug, Clone)]
pub struct CpuMaterial {
    pub diffuse: Color,
    pub emissive: f32,
    pub shininess: f32,
}

impl Default for CpuMaterial {
    fn default() -> Self {
        Self {
            diffuse: Color::WHITE,
            emissive: 0.0,
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
