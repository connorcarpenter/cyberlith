use math::*;

use render_api::components::PointLight;

use crate::{core::Program, renderer::Light};

impl Light for PointLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
        "
            uniform vec3 color{};
            uniform vec3 attenuation{};
            uniform vec3 position{};

            vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness)
            {{
                vec3 light_direction = position{} - position;
                float distance = length(light_direction);
                light_direction = light_direction / distance;

                vec3 light_color = attenuate(color{}, attenuation{}, distance);
                return calculate_light(light_color, light_direction, surface_color, view_direction, normal, metallic, roughness);
            }}
        
        ", i, i, i, i, i, i, i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        program.use_uniform(
            &format!("color{}", i),
            self.color.to_vec3() * self.intensity,
        );
        program.use_uniform(
            &format!("attenuation{}", i),
            Vec3::new(
                self.attenuation.constant,
                self.attenuation.linear,
                self.attenuation.quadratic,
            ),
        );
        program.use_uniform(&format!("position{}", i), self.position);
    }
}
