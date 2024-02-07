use std::hash::{Hash, Hasher};

use storage::AssetHash;

use crate::{base::Color};

#[derive(Debug, Clone)]
pub struct CpuMaterial {
    pub diffuse: Color,
    pub emissive: f32,
    // see https://learnopengl.com/Lighting/Basic-Lighting ... this value is inversed. larger values = smaller shine
    pub shine_size: f32,
    pub shine_amount: f32,
}

impl AssetHash<CpuMaterial> for CpuMaterial {}

impl Hash for CpuMaterial {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.diffuse.hash(state);
        self.emissive.to_bits().hash(state);
        self.shine_size.to_bits().hash(state);
    }
}

impl Default for CpuMaterial {
    fn default() -> Self {
        Self {
            diffuse: Color::WHITE,
            emissive: 0.0,
            shine_size: 32.0,
            shine_amount: 0.5,
        }
    }
}

impl CpuMaterial {
    pub fn new(diffuse: Color, emissive: f32, shine_size: f32, shine_amount: f32) -> Self {
        Self {
            diffuse,
            emissive,
            shine_size,
            shine_amount,
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
