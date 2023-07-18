use std::f32::consts::PI;

use math::{Quat, Vec2};

use crate::components::Transform;

pub fn set_line_transform(transform: &mut Transform, start: &Vec2, end: &Vec2) {
    let angle = angle_between(start, end);
    transform.translation.x = (start.x + end.x) / 2.0;
    transform.translation.y = (start.y + end.y) / 2.0;
    transform.rotation = Quat::from_rotation_z(angle);
    transform.scale.x = start.distance(*end) / 2.0;
    transform.scale.y = 1.0;
}

fn angle_between(a: &Vec2, b: &Vec2) -> f32 {
    let c = Vec2::new(a.x - b.x, a.y - b.y);
    let angle = c.y.atan2(c.x);
    angle + if angle < 0.0 { 2.0 * PI } else { 0.0 }
}