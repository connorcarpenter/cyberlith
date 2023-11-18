use std::hash::{Hash, Hasher};

use crate::{base::Color, AssetHash};

///
/// A light which shines on all surfaces.
/// Can be uniform (a light that shines equally on any surface) or calculated from an environment map using the [CpuTextureCube] struct.
///
#[derive(Clone, Debug, Hash)]
pub struct AmbientLight {
    pub color: AmbientLightColor,
}

impl AssetHash<AmbientLight> for AmbientLight {}

impl AmbientLight {
    pub fn none() -> Self {
        Self {
            color: AmbientLightColor::new(0.0, Color::WHITE),
        }
    }

    /// Constructs an ambient light that shines equally on all surfaces.
    pub fn new(intensity: f32, color: Color) -> Self {
        Self {
            color: AmbientLightColor::new(intensity, color),
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: AmbientLightColor::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AmbientLightColor {
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
}

impl Hash for AmbientLightColor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.intensity.to_bits().hash(state);
        self.color.hash(state);
    }
}

impl Default for AmbientLightColor {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

impl AmbientLightColor {
    pub fn new(intensity: f32, color: Color) -> Self {
        Self { intensity, color }
    }
}
