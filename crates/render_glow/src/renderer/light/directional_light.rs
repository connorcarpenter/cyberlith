use render_api::components::DirectionalLight;

use crate::core::Program;
use crate::renderer::Light;

impl Light for DirectionalLight {
    fn shader_source(&self, i: u32) -> String {
        format!(
                "
                    uniform vec3 color{};
                    uniform vec3 direction{};
        
                    vec3 calculate_light_{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness)
                    {{
                        return calculate_light(color{}, direction{}, surface_color, view_direction, normal, metallic, roughness);
                    }}
                
                ", i, i, i, i, i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        program.use_uniform(
            &format!("color{}", i),
            self.color.to_vec3() * self.intensity,
        );
        let mut light_dir = self.direction.normalize();
        light_dir *= -1.0;
        program.use_uniform(&format!("direction{}", i), light_dir);
    }
}
