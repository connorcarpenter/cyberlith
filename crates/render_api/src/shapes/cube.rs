use math::{Vec2, Vec3};

use crate::base::{Positions, TriMesh};

pub struct Cube {
    pub size: f32,
}

impl Cube {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl From<Cube> for TriMesh {
    fn from(cube: Cube) -> Self {
        let size = cube.size;
        let half_size = size / 2.0;
        let neg_half_size = -half_size;

        let tri_mesh = {
            let positions = vec![
                // Up
                Vec3::new(half_size, half_size, neg_half_size),
                Vec3::new(neg_half_size, half_size, neg_half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(neg_half_size, half_size, half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(neg_half_size, half_size, neg_half_size),
                // Down
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
                Vec3::new(half_size, neg_half_size, neg_half_size),
                Vec3::new(half_size, neg_half_size, half_size),
                Vec3::new(half_size, neg_half_size, half_size),
                Vec3::new(neg_half_size, neg_half_size, half_size),
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
                // Back
                Vec3::new(half_size, neg_half_size, neg_half_size),
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
                Vec3::new(half_size, half_size, neg_half_size),
                Vec3::new(neg_half_size, half_size, neg_half_size),
                Vec3::new(half_size, half_size, neg_half_size),
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
                // Front
                Vec3::new(neg_half_size, neg_half_size, half_size),
                Vec3::new(half_size, neg_half_size, half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(neg_half_size, half_size, half_size),
                Vec3::new(neg_half_size, neg_half_size, half_size),
                // Right
                Vec3::new(half_size, neg_half_size, neg_half_size),
                Vec3::new(half_size, half_size, neg_half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(half_size, half_size, half_size),
                Vec3::new(half_size, neg_half_size, half_size),
                Vec3::new(half_size, neg_half_size, neg_half_size),
                // Left
                Vec3::new(neg_half_size, half_size, neg_half_size),
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
                Vec3::new(neg_half_size, half_size, half_size),
                Vec3::new(neg_half_size, neg_half_size, half_size),
                Vec3::new(neg_half_size, half_size, half_size),
                Vec3::new(neg_half_size, neg_half_size, neg_half_size),
            ];
            let uvs = vec![
                // Up
                Vec2::new(0.25, 0.0),
                Vec2::new(0.25, 1.0 / 3.0),
                Vec2::new(0.5, 0.0),
                Vec2::new(0.5, 1.0 / 3.0),
                Vec2::new(0.5, 0.0),
                Vec2::new(0.25, 1.0 / 3.0),
                // Down
                Vec2::new(0.25, 2.0 / 3.0),
                Vec2::new(0.25, 1.0),
                Vec2::new(0.5, 1.0),
                Vec2::new(0.5, 1.0),
                Vec2::new(0.5, 2.0 / 3.0),
                Vec2::new(0.25, 2.0 / 3.0),
                // Back
                Vec2::new(0.0, 2.0 / 3.0),
                Vec2::new(0.25, 2.0 / 3.0),
                Vec2::new(0.0, 1.0 / 3.0),
                Vec2::new(0.25, 1.0 / 3.0),
                Vec2::new(0.0, 1.0 / 3.0),
                Vec2::new(0.25, 2.0 / 3.0),
                // Front
                Vec2::new(0.5, 2.0 / 3.0),
                Vec2::new(0.75, 2.0 / 3.0),
                Vec2::new(0.75, 1.0 / 3.0),
                Vec2::new(0.75, 1.0 / 3.0),
                Vec2::new(0.5, 1.0 / 3.0),
                Vec2::new(0.5, 2.0 / 3.0),
                // Right
                Vec2::new(1.0, 2.0 / 3.0),
                Vec2::new(1.0, 1.0 / 3.0),
                Vec2::new(0.75, 1.0 / 3.0),
                Vec2::new(0.75, 1.0 / 3.0),
                Vec2::new(0.75, 2.0 / 3.0),
                Vec2::new(1.0, 2.0 / 3.0),
                // Left
                Vec2::new(0.25, 1.0 / 3.0),
                Vec2::new(0.25, 2.0 / 3.0),
                Vec2::new(0.5, 1.0 / 3.0),
                Vec2::new(0.5, 2.0 / 3.0),
                Vec2::new(0.5, 1.0 / 3.0),
                Vec2::new(0.25, 2.0 / 3.0),
            ];
            let mut mesh = TriMesh {
                positions: Positions(positions),
                uvs: Some(uvs),
                ..Default::default()
            };
            mesh.compute_normals();
            mesh.compute_tangents();
            mesh
        };

        tri_mesh
    }
}
