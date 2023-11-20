use bevy_ecs::component::Component;

use math::Mat4;
use render_api::components::DirectionalLight;

use crate::core::{GpuDepthTexture2D, Program};
use crate::renderer::{Light, RenderObject};

///
/// A light which shines in the given direction.
/// The light will cast shadows if you [generate a shadow map](DirectionalLightImpl::generate_shadow_map).
///
#[derive(Component)]
pub struct DirectionalLightImpl {
    light: DirectionalLight,
}

impl From<&DirectionalLight> for DirectionalLightImpl {
    fn from(light: &DirectionalLight) -> Self {
        Self::new(light)
    }
}

impl DirectionalLightImpl {
    pub fn new(light: &DirectionalLight) -> Self {
        Self {
            light: light.clone(),
        }
    }

    pub fn use_light(&mut self, light: &DirectionalLight) {
        self.light.mirror(light);
    }
}

impl Light for DirectionalLightImpl {
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
            self.light.color.to_vec3() * self.light.intensity,
        );
        let mut light_dir = self.light.direction.normalize();
        light_dir *= -1.0;
        program.use_uniform(&format!("direction{}", i), light_dir);
    }
}
