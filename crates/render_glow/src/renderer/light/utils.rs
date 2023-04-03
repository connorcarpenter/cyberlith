use cgmath::*;

use render_api::base::*;

///
/// Returns shader source code with the function `calculate_lighting` which calculate the lighting contribution for the given lights and the given [LightingModel].
/// Use this if you want to implement a custom [Material](crate::renderer::Material) but use the default lighting calculations.
///
/// The shader function has the following signature:
/// ```no_rust
/// vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness, float occlusion)
/// ```
///
pub fn lights_shader_source(lights: &[&dyn Light], lighting_model: LightingModel) -> String {
    let mut shader_source = lighting_model_shader(lighting_model).to_string();
    shader_source.push_str(include_str!("../../core/shared.frag"));
    shader_source.push_str(include_str!("../light/shaders/light_shared.frag"));
    let mut dir_fun = String::new();
    for (i, light) in lights.iter().enumerate() {
        shader_source.push_str(&light.shader_source(i as u32));
        dir_fun.push_str(&format!("color += calculate_lighting{}(surface_color, position, normal, view_direction, metallic, roughness, occlusion);\n", i))
    }
    shader_source.push_str(&format!(
        "
            vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness, float occlusion)
            {{
                vec3 color = vec3(0.0, 0.0, 0.0);
                vec3 view_direction = normalize(camera_position - position);
                {}
                return color;
            }}
            ",
        &dir_fun
    ));
    shader_source
}

pub(crate) fn shadow_matrix(camera: &Camera) -> Mat4 {
    let bias_matrix = Mat4::new(
        0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.5, 0.5, 1.0,
    );
    bias_matrix * camera.projection() * camera.view()
}

pub(crate) fn compute_up_direction(direction: Vec3) -> Vec3 {
    if vec3(1.0, 0.0, 0.0).dot(direction).abs() > 0.9 {
        (vec3(0.0, 1.0, 0.0).cross(direction)).normalize()
    } else {
        (vec3(1.0, 0.0, 0.0).cross(direction)).normalize()
    }
}

use crate::renderer::Light;
use render_api::base::{LightingModel, NormalDistributionFunction};

pub(crate) fn lighting_model_shader(lighting_model: LightingModel) -> &'static str {
    match lighting_model {
        LightingModel::Phong => "#define PHONG",
        LightingModel::Blinn => "#define BLINN",
        LightingModel::Cook(normal, _) => match normal {
            NormalDistributionFunction::Blinn => "#define COOK\n#define COOK_BLINN\n",
            NormalDistributionFunction::Beckmann => "#define COOK\n#define COOK_BECKMANN\n",
            NormalDistributionFunction::TrowbridgeReitzGGX => "#define COOK\n#define COOK_GGX\n",
        },
    }
}
