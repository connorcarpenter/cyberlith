use bevy_ecs::component::Component;

use render_api::components::{AmbientLight, AmbientLightColor};

use crate::{core::*, renderer::*};

///
/// A light which shines on all surfaces.
/// Can be uniform (a light that shines equally on any surface) or calculated from an environment map using the [Environment] struct.
///
#[derive(Component)]
pub struct AmbientLightImpl {
    pub color: AmbientLightColor,
}

impl From<&AmbientLight> for AmbientLightImpl {
    fn from(ambient_light: &AmbientLight) -> Self {
        Self {
            color: ambient_light.color,
        }
    }
}

impl<'a> Light for AmbientLightImpl {
    fn shader_source(&self, i: u32) -> String {
        format!(
            "
                uniform vec3 ambientColor;
                vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness)
                {{
                    return ambientColor * mix(surface_color, vec3(0.0), metallic);
                }}

            ", i)
    }
    fn use_uniforms(&self, program: &Program, _i: u32) {
        program.use_uniform(
            "ambientColor",
            self.color.color.to_vec3() * self.color.intensity,
        );
    }
}

impl Default for AmbientLightImpl {
    fn default() -> Self {
        Self {
            color: AmbientLightColor::default(),
        }
    }
}
