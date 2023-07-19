use math::{Vec2, Vec3};

use crate::{assets::AssetHash, base::{CpuMesh, Indices, Positions}};

#[derive(Hash)]
pub struct Triangle {
    pub a: (i16, i16, i16),
    pub b: (i16, i16, i16),
    pub c: (i16, i16, i16),
}

impl AssetHash<CpuMesh> for Triangle {}

impl Triangle {
    pub fn new_3d(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self {
            a: (a.x as i16, a.y as i16, a.z as i16),
            b: (b.x as i16, b.y as i16, b.z as i16),
            c: (c.x as i16, c.y as i16, c.z as i16),
        }
    }

    pub fn new_2d(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self::new_3d(a.extend(0.0), b.extend(0.0), c.extend(0.0))
    }
}

impl From<Triangle> for CpuMesh {
    fn from(tri: Triangle) -> Self {
        let indices: Indices = Indices(Some(vec![0u16, 1, 2]));
        let positions = vec![
            Vec3::new(tri.a.0 as f32, tri.a.1 as f32, tri.a.2 as f32),
            Vec3::new(tri.b.0 as f32, tri.b.1 as f32, tri.b.2 as f32),
            Vec3::new(tri.c.0 as f32, tri.c.1 as f32, tri.c.2 as f32),
        ];
        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z];
        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
