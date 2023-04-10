use bevy_ecs::component::Component;

use render_api::AmbientLight;

use crate::{core::*, renderer::*};

///
/// A light which shines on all surfaces.
/// Can be uniform (a light that shines equally on any surface) or calculated from an environment map using the [Environment] struct.
///
#[derive(Component)]
pub struct AmbientLightImpl {
    /// The light shining from the environment. This is calculated based on an environment map.
    pub environment: Option<Environment>,
}

impl AmbientLightImpl {
    pub fn use_light(&mut self, light: &AmbientLight) {
        self.environment = light
            .environment
            .as_ref()
            .map(|environment_map| Environment::new(&environment_map.into()));
    }
}

impl From<&AmbientLight> for AmbientLightImpl {
    fn from(ambient_light: &AmbientLight) -> Self {
        Self {
            environment: ambient_light
                .environment
                .as_ref()
                .map(|environment_map| Environment::new(&environment_map.into())),
        }
    }
}

pub struct RenderAmbientLight<'a> {
    pub ambient_light: &'a AmbientLight,
    pub ambient_light_impl: &'a AmbientLightImpl,
}

impl<'a> RenderAmbientLight<'a> {
    pub fn new(ambient_light: &'a AmbientLight, ambient_light_impl: &'a AmbientLightImpl) -> Self {
        Self {
            ambient_light,
            ambient_light_impl,
        }
    }
}

impl<'a> Light for RenderAmbientLight<'a> {
    fn shader_source(&self, i: u32) -> String {
        if self.ambient_light_impl.environment.is_some() {
            format!(
            "
                uniform samplerCube irradianceMap;
                uniform samplerCube prefilterMap;
                uniform sampler2D brdfLUT;
                uniform vec3 ambientColor;
    
                vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)
                {{
                    vec3 N = normal;
                    vec3 V = view_direction;
                    vec3 R = reflect(-V, N); 
                    float NdV = max(0.001, dot(N, V));
                    
                    // calculate reflectance at normal incidence; if dia-electric (like plastic) use F0 
                    // of 0.04 and if it's a metal, use the albedo color as F0 (metallic workflow)    
                    vec3 F0 = mix(vec3(0.04), surface_color, metallic);
                    vec3 specular_fresnel = fresnel_schlick_roughness(F0, NdV, roughness);
                    vec3 diffuse_fresnel = 1.0 - specular_fresnel;

                    // Diffuse
                    vec3 irradiance = texture(irradianceMap, N).rgb;
                    vec3 diffuse = diffuse_fresnel * mix(surface_color, vec3(0.0), metallic) * irradiance;
                    
                    // sample both the pre-filter map and the BRDF lut and combine them together as per the Split-Sum approximation to get the IBL specular part.
                    const float MAX_REFLECTION_LOD = 4.0;
                    vec3 prefilteredColor = textureLod(prefilterMap, R,  roughness * MAX_REFLECTION_LOD).rgb;    
                    vec2 brdf  = texture(brdfLUT, vec2(NdV, roughness)).rg;
                    vec3 specular = prefilteredColor * (specular_fresnel * brdf.x + brdf.y);
    
                    return (diffuse + specular) * occlusion * ambientColor;
                }}
            
            ", i)
        } else {
            format!(
                "
                    uniform vec3 ambientColor;
                    vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)
                    {{
                        return occlusion * ambientColor * mix(surface_color, vec3(0.0), metallic);
                    }}
                
                ", i)
        }
    }
    fn use_uniforms(&self, program: &Program, _i: u32) {
        if let Some(ref environment) = self.ambient_light_impl.environment {
            program.use_texture_cube("irradianceMap", &environment.irradiance_map);
            program.use_texture_cube("prefilterMap", &environment.prefilter_map);
            program.use_texture("brdfLUT", &environment.brdf_map);
        }
        program.use_uniform(
            "ambientColor",
            self.ambient_light.color.to_vec3() * self.ambient_light.intensity,
        );
    }
}

impl Default for AmbientLightImpl {
    fn default() -> Self {
        Self { environment: None }
    }
}
