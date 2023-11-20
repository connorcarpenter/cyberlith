use crate::renderer::Light;

///
/// Returns shader source code with the function `calculate_lighting` which calculate the lighting contribution for the given lights and the given [LightingModel].
/// Use this if you want to implement a custom [Material](crate::renderer::Material) but use the default lighting calculations.
///
/// The shader function has the following signature:
/// ```no_rust
/// vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness)
/// ```
///
pub fn lights_shader_source(lights: &[&dyn Light]) -> String {
    let mut shader_source = String::new();
    shader_source.push_str(include_str!("../../core/shared.frag"));
    shader_source.push_str(include_str!("../light/shaders/light_shared.frag"));
    let mut dir_fun = String::new();
    for (i, light) in lights.iter().enumerate() {
        shader_source.push_str(&light.shader_source(i as u32));
        dir_fun.push_str(&format!("color += calculate_lighting{}(surface_color, position, normal, view_direction, metallic, roughness);\n", i))
    }
    shader_source.push_str(&format!(
        "
            vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness)
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

// pub(crate) fn shadow_matrix(
//     camera: &Camera,
//     projection: &Projection,
//     transform: &Transform,
// ) -> Mat4 {
//     let bias_matrix = Mat4::from_cols(
//         Vec4::new(0.5, 0.0, 0.0, 0.0),
//         Vec4::new(0.0, 0.5, 0.0, 0.0),
//         Vec4::new(0.0, 0.0, 0.5, 0.0),
//         Vec4::new(0.5, 0.5, 0.5, 1.0),
//     );
//     bias_matrix
//         * projection.projection_matrix(&camera.viewport_or_default())
//         * transform.compute_matrix()
// }

// pub(crate) fn compute_up_direction(direction: Vec3) -> Vec3 {
//     if Vec3::X.dot(direction).abs() > 0.9 {
//         (Vec3::Y.cross(direction)).normalize()
//     } else {
//         (Vec3::X.cross(direction)).normalize()
//     }
// }
