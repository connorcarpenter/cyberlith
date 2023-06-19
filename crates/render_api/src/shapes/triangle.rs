use math::{Vec2, Vec3};

use crate::base::{Indices, Positions, TriMesh};

pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    pub fn new_3d(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    pub fn new_2d(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self::new_3d(a.extend(0.0), b.extend(0.0), c.extend(0.0))
    }
}

impl From<Triangle> for TriMesh {
    fn from(tri: Triangle) -> Self {
        let indices: Indices = Indices(Some(vec![0u16, 1, 2]));
        let positions = vec![
            tri.a,
            tri.b,
            tri.c,
        ];
        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z];
        let uvs = vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
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
