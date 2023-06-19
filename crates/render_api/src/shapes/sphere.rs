use math::{Vec2, Vec3, Vec4};

use crate::{
    base::{Indices, Positions, TriMesh},
    components::Transform,
    shapes::Rectangle,
};

pub struct Sphere {
    pub angle_subdivisions: u32,
}

impl Sphere {
    pub fn new(angle_subdivisions: u32) -> Self {
        Self { angle_subdivisions }
    }
}

impl From<Sphere> for TriMesh {
    fn from(sphere: Sphere) -> Self {
        let angle_subdivisions = sphere.angle_subdivisions;

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        positions.push(Vec3::Z);
        normals.push(Vec3::Z);

        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push(0);
            indices.push((1 + j) as u16);
            indices.push((1 + j1) as u16);
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
                    indices.push((i0 + j) as u16);
                    indices.push((i1 + j1) as u16);
                    indices.push((i0 + j1) as u16);
                    indices.push((i1 + j1) as u16);
                    indices.push((i0 + j) as u16);
                    indices.push((i1 + j) as u16);
                }
            }
        }
        positions.push(Vec3::new(0.0, 0.0, -1.0));
        normals.push(Vec3::new(0.0, 0.0, -1.0));

        let i = 1 + (angle_subdivisions - 2) * angle_subdivisions * 2;
        for j in 0..angle_subdivisions * 2 {
            let j1 = (j + 1) % (angle_subdivisions * 2);
            indices.push((i + j) as u16);
            indices.push(((angle_subdivisions - 1) * angle_subdivisions * 2 + 1) as u16);
            indices.push((i + j1) as u16);
        }

        TriMesh {
            indices: Indices(Some(indices)),
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
