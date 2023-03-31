use crate::base::Color;

use std::default::Default;

use bevy_ecs::{bundle::Bundle, component::Component};

use crate::components::light::Attenuation;
use crate::Transform;

#[derive(Bundle, Default)]
pub struct PointLightBundle {
    pub point_light: PointLight,
    pub transform: Transform,
}

///
/// A light which shines from the given position in all directions.
///
#[derive(Component, Default)]
pub struct PointLight {
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
    /// The [Attenuation] of the light.
    pub attenuation: Attenuation,
}

impl PointLight {
    /// Constructs a new point light.
    pub fn new(intensity: f32, color: Color, attenuation: Attenuation) -> PointLight {
        PointLight {
            intensity,
            color,
            attenuation,
        }
    }
}
