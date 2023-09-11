use std::default::Default;

use math::Vec3;

use bevy_ecs::component::Component;

use crate::base::{Attenuation, Color};

///
/// A light which shines from the given position in all directions.
///
#[derive(Component, Copy, Clone)]
pub struct PointLight {
    pub position: Vec3,
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
    /// The [Attenuation] of the light.
    pub attenuation: Attenuation,
}

impl PointLight {
    /// Constructs a new point light.
    pub fn new(
        position: Vec3,
        intensity: f32,
        color: Color,
        attenuation: Attenuation,
    ) -> PointLight {
        PointLight {
            position,
            intensity,
            color,
            attenuation,
        }
    }
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Color::WHITE,
            intensity: 1.0,
            attenuation: Attenuation::default(),
        }
    }
}
