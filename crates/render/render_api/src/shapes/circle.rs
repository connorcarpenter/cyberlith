use math::Vec3;
use storage::StorageHash;

use crate::base::CpuMesh;

// Circle
#[derive(Hash)]
pub struct Circle {
    pub angle_subdivisions: u16,
}

impl StorageHash<CpuMesh> for Circle {}

impl Circle {
    pub fn new(angle_subdivisions: u16) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<Circle> for CpuMesh {
    fn from(circle: Circle) -> Self {
        let angle_subdivisions = circle.angle_subdivisions;

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            positions.push(Vec3::new(angle.cos(), angle.sin(), 0.0));
        }

        for j in 0..angle_subdivisions {
            indices.push(0 as usize);
            indices.push(((j + 1) % angle_subdivisions) as usize);
            indices.push(j as usize);
        }

        CpuMesh::from_indices(&positions, &indices)
    }
}

// HollowCircle
#[derive(Hash)]
pub struct HollowCircle {
    pub angle_subdivisions: u16,
}

impl StorageHash<CpuMesh> for HollowCircle {}

impl HollowCircle {
    pub fn new(angle_subdivisions: u16) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<HollowCircle> for CpuMesh {
    fn from(circle: HollowCircle) -> Self {
        let radius = 1.0;
        let line_thickness_half = 0.1;

        let angle_subdivisions = circle.angle_subdivisions;

        let outer_radius = radius + line_thickness_half;
        let inner_radius = radius - line_thickness_half;

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

            // inner
            positions.push(Vec3::new(
                angle.cos() * inner_radius,
                angle.sin() * inner_radius,
                0.0,
            ));
            normals.push(Vec3::Z);

            // outer
            positions.push(Vec3::new(
                angle.cos() * outer_radius,
                angle.sin() * outer_radius,
                0.0,
            ));
            normals.push(Vec3::Z);
        }

        for j in 0u16..angle_subdivisions {
            let a = j * 2;
            let b = j * 2 + 1;
            let next_j = (j + 1) % angle_subdivisions;
            let c = next_j * 2;
            let d = next_j * 2 + 1;

            indices.push(a as usize);
            indices.push(c as usize);
            indices.push(b as usize);

            indices.push(c as usize);
            indices.push(d as usize);
            indices.push(b as usize);
        }
        CpuMesh::from_indices(&positions, &indices)
    }
}
