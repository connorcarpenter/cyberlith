use math::Vec3;
use render_api::components::Transform;

pub fn set_3d_line_transform(transform: &mut Transform, start: Vec3, end: Vec3, angle_opt: Option<f32>) {
    transform.translation = start;
    let translation_diff = end - start;

    if translation_diff.x == 0.0 && translation_diff.z == 0.0 {
        transform.look_to(translation_diff, Vec3::Z);
    } else {
        transform.look_to(translation_diff, Vec3::Y);
    }

    if let Some(angle) = angle_opt {
        transform.rotate_axis(translation_diff, angle);
    }

    transform.scale.z = start.distance(end);
}
