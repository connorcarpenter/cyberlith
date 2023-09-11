use bevy_ecs::component::Component;

use crate::base::{Color, CpuTextureCube};

///
/// A light which shines on all surfaces.
/// Can be uniform (a light that shines equally on any surface) or calculated from an environment map using the [CpuTextureCube] struct.
///
#[derive(Component)]
pub struct AmbientLight {
    pub color: AmbientLightColor,
    /// The light shining from the environment. This is calculated based on an environment map.
    pub environment: Option<CpuTextureCube>,
}

impl AmbientLight {
    pub fn none() -> Self {
        Self {
            color: AmbientLightColor::new(0.0, Color::WHITE),
            environment: None,
        }
    }

    /// Constructs an ambient light that shines equally on all surfaces.
    pub fn new(intensity: f32, color: Color) -> Self {
        Self {
            color: AmbientLightColor::new(intensity, color),
            environment: None,
        }
    }

    /// Constructs an ambient light that shines based on the given environment map.
    pub fn new_with_environment(
        intensity: f32,
        color: Color,
        environment_map: CpuTextureCube,
    ) -> Self {
        Self {
            color: AmbientLightColor::new(intensity, color),
            environment: Some(environment_map),
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: AmbientLightColor::default(),
            environment: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AmbientLightColor {
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
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
