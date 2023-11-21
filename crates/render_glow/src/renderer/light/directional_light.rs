use render_api::components::DirectionalLight;

use crate::core::Program;
use crate::renderer::Light;

impl Light for DirectionalLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
                "
                    uniform vec3 light_color_{};
                    uniform vec3 light_dir_{};
        
                    vec3 calculate_light_{}(vec3 position, vec3 normal, vec3 view_direction, vec3 material_color, float material_shininess)
                    {{
                        vec3 light_direction = vec3(light_dir_{}.x, -light_dir_{}.z, -light_dir_{}.y);
                        return calculate_light(light_color_{}, light_direction, view_direction, normal, material_color, material_shininess);
                    }}
                
                ", i, i, i, i, i, i, i,
        )
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        program.use_uniform(
            &format!("light_color_{}", i),
            self.color.to_vec3() * self.intensity,
        );
        let mut light_dir = self.direction.normalize();
        light_dir *= -1.0;
        program.use_uniform(&format!("light_dir_{}", i), light_dir);
    }
}
