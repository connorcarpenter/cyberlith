use math::Vec3;

use crate::base::{CpuMesh, Indices, Positions};

pub struct Cone {
    pub angle_subdivisions: u32,
}

impl Cone {
    pub fn new(angle_subdivisions: u32) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<Cone> for CpuMesh {
    fn from(cone: Cone) -> Self {
        // TODO: this method creates `angle_subdivisions` + 1 vertices at the top of the cone
        // change this to a single vertex at the top of the cone
        let angle_subdivisions = cone.angle_subdivisions;
        let length_subdivisions = 1;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for i in 0..length_subdivisions + 1 {
            let x = i as f32 / length_subdivisions as f32;
            for j in 0..angle_subdivisions {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

                positions.push(Vec3::new(
                    x,
                    angle.cos() * (1.0 - x),
                    angle.sin() * (1.0 - x),
                ));
            }
        }
        for i in 0..length_subdivisions {
            for j in 0..angle_subdivisions {
                indices.push((i * angle_subdivisions + j) as u16);
                indices.push((i * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);

                indices.push((i * angle_subdivisions + j) as u16);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
                indices.push(((i + 1) * angle_subdivisions + j) as u16);
            }
        }
        let mut mesh = Self {
            positions: Positions(positions),
            indices: Indices(Some(indices)),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh
    }
}
