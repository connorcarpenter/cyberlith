use math::{Vec2, Vec3};

use crate::base::{CpuMesh, Indices, Positions};

pub struct Square;

impl From<Square> for CpuMesh {
    fn from(_square: Square) -> Self {
        let half_width = 1.0;
        let half_height = 1.0;
        let neg_half_width = -1.0;
        let neg_half_height = -1.0;

        let indices: Indices = Indices(Some(vec![0u16, 1, 2, 2, 3, 0]));
        let positions = vec![
            Vec3::new(neg_half_width, neg_half_height, 0.0),
            Vec3::new(half_width, neg_half_height, 0.0),
            Vec3::new(half_width, half_height, 0.0),
            Vec3::new(neg_half_width, half_height, 0.0),
        ];
        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z];
        let uvs = vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];
        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            uvs: Some(uvs),
            ..Default::default()
        }
    }
}
