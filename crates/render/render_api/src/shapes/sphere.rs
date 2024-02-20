use math::Vec3;
use storage::AssetHash;

use crate::base::CpuMesh;

#[derive(Hash)]
pub struct Sphere {
    pub angle_subdivisions: u16,
}

impl Sphere {
    pub fn new(angle_subdivisions: u16) -> Self {
        Self { angle_subdivisions }
    }
}

impl AssetHash<CpuMesh> for Sphere {}

impl From<Sphere> for CpuMesh {
    fn from(sphere: Sphere) -> Self {
        let angle_subdivisions = sphere.angle_subdivisions;

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        positions.push(Vec3::Z);
        normals.push(Vec3::Z);

        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push(0 as usize);
            indices.push((1 + j) as usize);
            indices.push((1 + j1) as usize);
        }

        for i in 0..angle_subdivisions - 1 {
            let theta = std::f32::consts::PI * (i + 1) as f32 / angle_subdivisions as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let i0 = 1 + i * angle_subdivisions * 2;
            let i1 = 1 + (i + 1) * angle_subdivisions * 2;

            for j in 0..angle_subdivisions * 2 {
                let phi = std::f32::consts::PI * j as f32 / angle_subdivisions as f32;
                let x = sin_theta * phi.cos();
                let y = sin_theta * phi.sin();
                let z = cos_theta;
                positions.push(Vec3::new(x, y, z));
                normals.push(Vec3::new(x, y, z));

                if i != angle_subdivisions - 2 {
                    let j1 = (j + 1) % (angle_subdivisions * 2);
                    indices.push((i0 + j) as usize);
                    indices.push((i1 + j1) as usize);
                    indices.push((i0 + j1) as usize);
                    indices.push((i1 + j1) as usize);
                    indices.push((i0 + j) as usize);
                    indices.push((i1 + j) as usize);
                }
            }
        }
        positions.push(Vec3::new(0.0, 0.0, -1.0));
        normals.push(Vec3::new(0.0, 0.0, -1.0));

        let i = 1 + (angle_subdivisions - 2) * angle_subdivisions * 2;
        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push((i + j) as usize);
            indices.push(((angle_subdivisions - 1) * angle_subdivisions * 2 + 1) as usize);
            indices.push((i + j1) as usize);
        }

        CpuMesh::from_indices(&positions, &indices)
    }
}
