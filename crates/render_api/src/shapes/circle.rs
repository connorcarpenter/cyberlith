use math::Vec3;

use crate::{
    assets::AssetHash,
    base::{CpuMesh, Indices, Positions},
};

#[derive(Hash)]
pub struct Circle {
    pub angle_subdivisions: u32,
}

impl AssetHash<CpuMesh> for Circle {}

impl Circle {
    pub fn new(angle_subdivisions: u32) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<Circle> for CpuMesh {
    fn from(circle: Circle) -> Self {
        let angle_subdivisions = circle.angle_subdivisions;

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(Vec3::new(angle.cos(), angle.sin(), 0.0));
            normals.push(Vec3::Z);
        }

        for j in 0..angle_subdivisions {
            indices.push(0);
            indices.push(j as u16);
            indices.push(((j + 1) % angle_subdivisions) as u16);
        }
        CpuMesh {
            indices: Indices(Some(indices)),
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
