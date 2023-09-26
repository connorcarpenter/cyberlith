use math::{Quat, Vec3};
use render_api::components::Transform;

pub fn set_3d_line_transform(
    transform: &mut Transform,
    start: Vec3,
    end: Vec3,
    angle_opt: Option<f32>,
) {
    transform.translation = start;
    let translation_diff = end - start;
    let target_direction = translation_diff.normalize();

    if translation_diff.x == 0.0 && translation_diff.y == 0.0 {
        transform.look_to(translation_diff, Vec3::Y);
    } else {
        let base_direction = Vec3::Z;
        let axis_of_rotation = base_direction.cross(target_direction).normalize();
        let angle = base_direction.angle_between(target_direction);
        let rotation_angle = Quat::from_axis_angle(axis_of_rotation, angle);
        transform.rotation = rotation_angle;
    }

    if let Some(angle) = angle_opt {
        transform.rotate_axis(target_direction, angle);
    }

    transform.scale.z = start.distance(end);
}
