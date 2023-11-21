use math::*;

use render_api::components::PointLight;

use crate::{core::Program, renderer::Light};

impl Light for PointLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
        "
            uniform vec3 light_color_{};
            uniform vec3 light_attenuation_{};
            uniform vec3 light_position_{};

            vec3 calculate_light_{}(vec3 position, vec3 normal, vec3 view_direction, vec3 material_color, float material_shininess)
            {{
                vec3 light_direction = light_position_{} - position;
                float distance = length(light_direction);
                light_direction = light_direction / distance;
                light_direction = vec3(light_direction.x, -light_direction.z, -light_direction.y);

                vec3 light_color = attenuate(light_color_{}, light_attenuation_{}, distance);
                return calculate_light(light_color, light_direction, view_direction, normal, material_color, material_shininess);
            }}
        
        ", i, i, i, i, i, i, i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        program.use_uniform(
            &format!("light_color_{}", i),
            self.color.to_vec3() * self.intensity,
        );
        program.use_uniform(
            &format!("light_attenuation_{}", i),
            Vec3::new(
                self.attenuation.constant,
                self.attenuation.linear,
                self.attenuation.quadratic,
            ),
        );
        program.use_uniform(&format!("light_position_{}", i), self.position);
    }
}
