use render_api::components::AmbientLight;

use crate::{core::*, renderer::*};

impl<'a> Light for AmbientLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
            "
                uniform vec3 light_color_{};
                vec3 calculate_single_light_{}(vec3 position, vec3 normal, vec3 view_direction, vec3 material_color, vec2 material_shine)
                {{
                    return light_color_{} * material_color;
                }}

            ", i, i, i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        program.use_uniform(
            &format!("light_color_{}", i),
            self.color.color.to_vec3() * self.color.intensity,
        );
    }
}
