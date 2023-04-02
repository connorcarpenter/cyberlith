use bevy_ecs::system::Resource;

use crate::base::{Color, TextureCubeMap};

///
/// A light which shines on all surfaces.
/// Can be uniform (a light that shines equally on any surface) or calculated from an environment map using the [TextureCubeMap] struct.
///
#[derive(Resource)]
pub struct AmbientLight {
    /// The intensity of the light. This allows for higher intensity than 1 which can be used to simulate high intensity light sources like the sun.
    pub intensity: f32,
    /// The base color of the light.
    pub color: Color,
    /// The light shining from the environment. This is calculated based on an environment map.
    pub environment: Option<TextureCubeMap>,
}

impl AmbientLight {
    /// Constructs an ambient light that shines equally on all surfaces.
    pub fn new(intensity: f32, color: Color) -> Self {
        Self {
            intensity,
            color,
            environment: None,
        }
    }

    /// Constructs an ambient light that shines based on the given environment map.
    pub fn new_with_environment(
        intensity: f32,
        color: Color,
        environment_map: TextureCubeMap,
    ) -> Self {
        Self {
            intensity,
            color,
            environment: Some(environment_map),
        }
    }
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            environment: None,
        }
    }
}
