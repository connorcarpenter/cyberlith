use std::f32::consts::PI;

use math::{Quat, Vec2, Vec3};

use crate::{
    assets::AssetHash,
    base::{CpuMesh, Indices, Positions},
    components::Transform,
};

pub fn set_2d_line_transform(transform: &mut Transform, start: Vec2, end: Vec2, depth: f32) {
    let angle = angle_between(&start, &end);
    set_2d_line_transform_from_angle(transform, start, angle, start.distance(end), depth);
}

pub fn set_2d_line_transform_from_angle(
    transform: &mut Transform,
    start: Vec2,
    angle: f32,
    length: f32,
    depth: f32,
) {
    transform.translation.x = start.x;
    transform.translation.y = start.y;
    transform.translation.z = depth;
    transform.rotation = Quat::from_rotation_z(normalize_angle(angle));
    transform.scale.x = length;
}

pub fn get_2d_line_transform_endpoint(transform: &Transform) -> Vec2 {
    let unit_vector =
        transform.rotation.mul_vec3(Vec3::X).normalize().truncate() * transform.scale.x;
    return Vec2::new(
        transform.translation.x + unit_vector.x,
        transform.translation.y + unit_vector.y,
    );
}

pub fn distance_to_2d_line(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_diff = line_end - line_start;
    let point_diff = point - line_start;

    let c1 = point_diff.dot(line_diff);
    if c1 <= 0.0 {
        return point.distance(line_start);
    }

    let c2 = line_diff.dot(line_diff);
    if c2 <= c1 {
        return point.distance(line_end);
    }

    let b = c1 / c2;
    let pb = line_start + (line_diff * b);
    return point.distance(pb);
}

pub fn angle_between(a: &Vec2, b: &Vec2) -> f32 {
    let c = Vec2::new(b.x - a.x, b.y - a.y);
    let angle = c.y.atan2(c.x);
    angle + if angle < 0.0 { 2.0 * PI } else { 0.0 }
}

// radians
pub fn rotation_diff(a: f32, b: f32) -> f32 {
    // Normalize both angles to the [0, 2π] range
    let a_normalized = normalize_angle(a);
    let b_normalized = normalize_angle(b);

    // Calculate the absolute angular difference
    normalize_angle(a_normalized - b_normalized)
}

pub fn normalize_angle(angle: f32) -> f32 {
    let mut angle = angle;
    while angle < 0.0 {
        angle += 2.0 * PI;
    }
    while angle > 2.0 * PI {
        angle -= 2.0 * PI;
    }
    angle
}

#[derive(Hash)]
pub struct Line;

impl AssetHash<CpuMesh> for Line {}

impl From<Line> for CpuMesh {
    fn from(_line: Line) -> Self {

        let positions = vec![
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(1.0, -0.5, 0.0),
            Vec3::new(1.0, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
        ];

        //let indices: Indices = Indices(Some(vec![0u16, 1, 2, 2, 3, 0]));
        let indices: Indices = Indices(Some(vec![0u16, 2, 1, 2, 0, 3]));

        let normals = vec![Vec3::Z; 4];

        Self {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
