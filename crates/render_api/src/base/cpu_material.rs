use std::hash::{Hash, Hasher};

use crate::{AssetHash, base::Color};


#[derive(Debug, Clone)]
pub struct CpuMaterial {
    pub diffuse: Color,
    pub emissive: f32,
    pub shininess: f32,
}

impl AssetHash<CpuMaterial> for CpuMaterial {}

impl Hash for CpuMaterial {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.diffuse.hash(state);
        self.emissive.to_bits().hash(state);
        self.shininess.to_bits().hash(state);
    }
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

impl CpuMaterial {
    pub fn new(diffuse: Color, emissive: f32, shininess: f32) -> Self {
        Self {
            diffuse,
            emissive,
            shininess,
        }
    }
}

// impl From<Color> for CpuMaterial {
//     fn from(color: Color) -> Self {
//         Self {
//             diffuse: color,
//             ..Default::default()
//         }
//     }
// }
