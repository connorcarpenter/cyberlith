use bevy_ecs::component::Component;

use crate::base::{Color, Vec3};

///
/// A light which shines in the given direction.
/// The light will cast shadows if you [generate a shadow map](DirectionalLight::generate_shadow_map).
///
#[derive(Component)]
pub struct DirectionalLight {
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
    /// The direction the light shines.
    pub direction: Vec3,
}

impl DirectionalLight {
    /// Creates a new directional light.
    pub fn new(intensity: f32, color: Color, direction: &Vec3) -> DirectionalLight {
        DirectionalLight {
            intensity,
            color,
            direction: *direction,
        }
    }
}
