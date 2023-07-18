use std::f32::consts::PI;

use math::{Quat, Vec2, Vec3};

use crate::{assets::AssetHash, base::{CpuMesh, Indices, Positions}, components::Transform};

pub fn set_line_transform(transform: &mut Transform, start: &Vec2, end: &Vec2) {
    let angle = angle_between(start, end);
    transform.translation.x = start.x;
    transform.translation.y = start.y;
    transform.rotation = Quat::from_rotation_z(angle);
    transform.scale.x = start.distance(*end);
    transform.scale.y = 1.0;
}

fn angle_between(a: &Vec2, b: &Vec2) -> f32 {
    let c = Vec2::new(b.x - a.x, b.y - a.y);
    let angle = c.y.atan2(c.x);
    angle + if angle < 0.0 { 2.0 * PI } else { 0.0 }
}

#[derive(Hash)]
pub struct Line;

impl Line {
    pub fn new() -> Self {
        Self
    }
}

impl AssetHash<CpuMesh> for Line {}

impl From<Line> for CpuMesh {
    fn from(_line: Line) -> Self {
        let indices: Indices = Indices(Some(vec![0u16, 1, 2, 2, 3, 0]));
        let positions = vec![
            Vec3::new(0.0, -0.5, 0.0),
            Vec3::new(1.0, -0.5, 0.0),
            Vec3::new(1.0, 0.5, 0.0),
            Vec3::new(0.0, 0.5, 0.0),
        ];
        let normals = vec![Vec3::Z; 4];
        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}