use math::Vec3;

use crate::base::CpuMesh;

pub struct Cylinder {
    pub angle_subdivisions: u32,
}

impl Cylinder {
    pub fn new(angle_subdivisions: u32) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<Cylinder> for CpuMesh {
    fn from(cylinder: Cylinder) -> Self {
        let angle_subdivisions = cylinder.angle_subdivisions;
        let length_subdivisions = 1;
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        for i in 0..length_subdivisions + 1 {
            let x = i as f32 / length_subdivisions as f32;
            for j in 0..angle_subdivisions {
                let angle = 2.0 * std::f32::consts::PI * j as f32 / angle_subdivisions as f32;

                positions.push(Vec3::new(x, angle.cos(), angle.sin()));
            }
        }
        for i in 0..length_subdivisions {
            for j in 0..angle_subdivisions {
                indices.push((i * angle_subdivisions + j) as usize);
                indices.push((i * angle_subdivisions + (j + 1) % angle_subdivisions) as usize);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as usize);

                indices.push((i * angle_subdivisions + j) as usize);
                indices.push(((i + 1) * angle_subdivisions + (j + 1) % angle_subdivisions) as usize);
                indices.push(((i + 1) * angle_subdivisions + j) as usize);
            }
        }
        CpuMesh::from_indices(&positions, &indices)
    }
}
