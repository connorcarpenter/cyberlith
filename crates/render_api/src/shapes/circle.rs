use math::Vec3;

use crate::base::{Indices, Positions, TriMesh};

pub struct Circle {
    pub radius: f32,
    pub angle_subdivisions: u32,
}

impl Circle {
    pub fn new(radius: f32, angle_subdivisions: u32) -> Self {
        Self { radius, angle_subdivisions }
    }
}

impl From<Circle> for TriMesh {
    fn from(circle: Circle) -> Self {
        let angle_subdivisions = circle.angle_subdivisions;
        let radius = circle.radius;

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(Vec3::new(angle.cos() * radius, angle.sin() * radius, 0.0));
            normals.push(Vec3::Z);
        }

        for j in 0..angle_subdivisions {
            indices.push(0);
            indices.push(j as u16);
            indices.push(((j + 1) % angle_subdivisions) as u16);
        }
        TriMesh {
            indices: Indices(Some(indices)),
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
