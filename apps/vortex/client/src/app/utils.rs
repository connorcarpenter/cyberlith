use math::Vec3;
use render_api::components::Transform;

pub fn set_3d_line_transform(transform: &mut Transform, start: Vec3, end: Vec3) {
    transform.translation = start;
    let translation_diff = end - start;
    if translation_diff.x == 0.0 && translation_diff.z == 0.0 {
        transform.look_at(end, Vec3::Z);
    } else {
        transform.look_at(end, Vec3::Y);
    }

    transform.scale = Vec3::new(1.0, 1.0, start.distance(end));
}
