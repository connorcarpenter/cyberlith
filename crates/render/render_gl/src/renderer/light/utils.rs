use crate::renderer::Light;

pub fn lights_shader_source(lights: &[&dyn Light]) -> String {
    let mut shader_source = String::new();
    shader_source.push_str(include_str!("../../shaders/shared.vert"));
    shader_source.push_str(include_str!("../../shaders/light_shared.vert"));
    let mut dir_fun = String::new();
    for (i, light) in lights.iter().enumerate() {
        shader_source.push_str(&light.shader_source(i as u32));
        dir_fun.push_str(&format!("color += calculate_single_light_{}(position, normal, view_direction, material_color, material_shine);\n", i))
    }
    shader_source.push_str(&format!(
        "
            vec3 calculate_total_light(vec3 camera_position, vec3 position, vec3 normal, vec3 material_color, vec2 material_shine)
            {{
                vec3 color = vec3(0.0, 0.0, 0.0);
                vec3 view_direction = normalize(camera_position - position);

                // convert from right-handed y-up to left-handed z-up
                view_direction = vec3(view_direction.x, -view_direction.z, -view_direction.y);
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
