//!
//! Contain material asset definitions.
//!

use crate::base::Color;

///
/// A CPU-side version of a material used for physically based rendering (PBR).
///
#[derive(Debug, Clone)]
pub struct CpuMaterial {
    /// Name. Used for matching geometry and material.
    pub name: String,
    /// Albedo base color, also called diffuse color. Assumed to be in linear color space.
    pub albedo: Color,
    /// A value in the range `[0..1]` specifying how metallic the material is.
    pub metallic: f32,
    /// A value in the range `[0..1]` specifying how rough the material surface is.
    pub roughness: f32,
    /// Color of light shining from an object.
    pub emissive: Color,
    /// The index of refraction for this material
    pub index_of_refraction: f32,
    /// A value in the range `[0..1]` specifying how transmissive the material surface is.
    pub transmission: f32,
}

impl Default for CpuMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            albedo: Color::WHITE,
            metallic: 0.0,
            roughness: 1.0,
            emissive: Color::BLACK,
            index_of_refraction: 1.5,
            transmission: 0.0,
        }
    }
}

impl From<Color> for CpuMaterial {
    fn from(color: Color) -> Self {
        Self {
            albedo: color,
            ..Default::default()
        }
    }
}
