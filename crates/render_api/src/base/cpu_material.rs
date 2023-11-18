//!
//! Contain material asset definitions.
//!

use crate::base::Color;

/// Lighting models which specify how the lighting is computed when rendering a material.
/// This is a trade-off between how fast the computations are versus how physically correct they look.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightingModel {
    /// Phong lighting model.
    /// The fastest lighting model to calculate.
    Phong,
    /// Blinn lighting model.
    /// Almost as fast as Phong and has less artifacts.
    Blinn,
    /// Cook-Torrance lighting model with the given normal distribution and geometry functions.
    /// The most physically correct lighting model but also the most expensive.
    Cook(NormalDistributionFunction, GeometryFunction),
}

/// The geometry function used in a Cook-Torrance lighting model.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum GeometryFunction {
    SmithSchlickGGX,
}

/// The normal distribution function used in a Cook-Torrance lighting model.
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum NormalDistributionFunction {
    Blinn,
    Beckmann,
    TrowbridgeReitzGGX,
}

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
    /// The lighting model used when rendering this material
    pub lighting_model: LightingModel,
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
            lighting_model: LightingModel::Phong,
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
