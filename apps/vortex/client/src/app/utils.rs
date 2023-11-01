use math::{quat_from_spin_direction, Vec3};
use render_api::components::Transform;

pub fn set_3d_line_transform(transform: &mut Transform, start: Vec3, end: Vec3, spin: f32) {
    transform.translation = start;
    let translation_diff = end - start;
    let target_direction = translation_diff.normalize();

    if translation_diff.x == 0.0 && translation_diff.y == 0.0 {
        transform.look_to(translation_diff, Vec3::Y);
    } else {
        let rotation_angle = quat_from_spin_direction(spin, Vec3::Z, target_direction);
        transform.rotation = rotation_angle;
    }

    transform.scale.z = start.distance(end);
}

pub fn transform_from_endpoints_and_spin(start: Vec3, end: Vec3, spin: f32) -> Transform {
    let mut output = Transform::default();
    output.translation = start;
    let translation_diff = end - start;
    let target_direction = translation_diff.normalize();

    if translation_diff.x == 0.0 && translation_diff.y == 0.0 {
        output.look_to(translation_diff, Vec3::Y);
    } else {
        let rotation_angle = quat_from_spin_direction(spin, Vec3::Z, target_direction);
        output.rotation = rotation_angle;
    }

    output
}