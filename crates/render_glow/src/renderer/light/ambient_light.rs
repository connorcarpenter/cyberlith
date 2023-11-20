use render_api::components::AmbientLight;

use crate::{core::*, renderer::*};

impl<'a> Light for AmbientLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
            "
                uniform vec3 ambientColor;
                vec3 calculate_light_{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness)
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
