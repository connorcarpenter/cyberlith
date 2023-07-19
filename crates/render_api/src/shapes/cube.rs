use math::Vec3;

use crate::{assets::AssetHash, base::{CpuMesh, Positions}};

#[derive(Hash)]
pub struct Cube;

impl AssetHash<CpuMesh> for Cube {}

impl From<Cube> for CpuMesh {
    fn from(_cube: Cube) -> Self {
        let half_size = 1.0;
        let neg_half_size = -1.0;

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
        let mut mesh = CpuMesh {
            positions: Positions(positions),
            ..Default::default()
        };
        mesh.compute_normals();
        mesh
    }
}
