use math::{Vec2, Vec3};

use crate::base::{Indices, Positions, TriMesh};

pub struct Rectangle {
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<Rectangle> for TriMesh {
    fn from(rect: Rectangle) -> Self {
        let half_width = rect.width / 2.0;
        let half_height = rect.height / 2.0;
        let neg_half_width = -half_width;
        let neg_half_height = -half_height;

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
        TriMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            uvs: Some(uvs),
            ..Default::default()
        }
    }
}
