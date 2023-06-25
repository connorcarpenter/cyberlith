use glam::{Mat4, Vec2, Vec3, Vec4};

pub fn convert_3d_to_2d(
    view_matrix: &Mat4,
    projection_matrix: &Mat4,
    viewport: &Vec2,
    point_3d: &Vec3,
) -> (Vec2, f32) {
    let viewport_width = viewport.x;
    let viewport_height = viewport.y;

    // Calculate the clip space coordinate
    let clip_space_coordinate = *projection_matrix * *view_matrix * point_3d.extend(1.0);

    // Normalize the clip space coordinate
    let clip_space_vec3 = Vec3::new(
        clip_space_coordinate.x,
        clip_space_coordinate.y,
        clip_space_coordinate.z,
    );
    let normalized_device_coordinate = clip_space_vec3 / clip_space_coordinate.w;

    // Convert the normalized device coordinate to screen space coordinate
    let screen_space_x = (normalized_device_coordinate.x + 1.0) * 0.5 * viewport_width;
    let screen_space_y = (1.0 - normalized_device_coordinate.y) * 0.5 * viewport_height;
    let screen_space_d = normalized_device_coordinate.z; // -1.0 -> 1.0 (near -> far)

    // The resulting screen space coordinates
    (Vec2::new(screen_space_x, screen_space_y), screen_space_d)
}

pub fn convert_2d_to_3d(
    view_matrix: &Mat4,
    projection_matrix: &Mat4,
    viewport: &Vec2,
    point_2d: &Vec2,
    point_depth: f32,
) -> Vec3 {
    let viewport_width = viewport.x;
    let viewport_height = viewport.y;

    // Convert the screen space coordinate to normalized device coordinate (NDC)
    let normalized_device_coordinate = Vec3::new(
        (2.0 * point_2d.x) / viewport_width - 1.0,
        1.0 - (2.0 * point_2d.y) / viewport_height,
        point_depth,
    );

    // Convert NDC to clip space
    let clip_space_coordinate = Vec4::new(
        normalized_device_coordinate.x,
        normalized_device_coordinate.y,
        normalized_device_coordinate.z,
        1.0,
    );

    // Inverse projection matrix
    let inv_projection_matrix = projection_matrix.inverse();

    // Inverse view matrix
    let inv_view_matrix = view_matrix.inverse();

    // Calculate the clip space coordinate
    let clip_space_vec4 = inv_projection_matrix * clip_space_coordinate;

    // Convert the clip space coordinate to world space coordinate
    let world_space_vec4 = inv_view_matrix * clip_space_vec4;

    // Return the resulting 3D world coordinate
    Vec3::new(world_space_vec4.x, world_space_vec4.y, world_space_vec4.z)
}
