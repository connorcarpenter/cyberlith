use bevy_log::info;
use math::{Vec2, Vec3};

use crate::{
    assets::AssetHash,
    base::{CpuMesh, Indices, Positions},
};

// Triangle
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

    pub fn new_2d_equilateral() -> Self {
        Self::new_2d(
            Vec2::new(0.0, -1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(1.0, 1.0),
        )
    }
}

impl From<Triangle> for CpuMesh {
    fn from(tri: Triangle) -> Self {
        let mut outer_a = Vec3::new(tri.a.0 as f32, tri.a.1 as f32, tri.a.2 as f32);
        let mut outer_b = Vec3::new(tri.b.0 as f32, tri.b.1 as f32, tri.b.2 as f32);
        let mut outer_c = Vec3::new(tri.c.0 as f32, tri.c.1 as f32, tri.c.2 as f32);

        let center = Vec3::new(
            (outer_a.x + outer_b.x + outer_c.x) / 3.0,
            (outer_a.y + outer_b.y + outer_c.y) / 3.0,
            (outer_a.z + outer_b.z + outer_c.z) / 3.0,
        );

        outer_a -= center;
        outer_b -= center;
        outer_c -= center;

        let positions = vec![outer_a, outer_b, outer_c];
        let indices: Indices = Indices(Some(vec![0u16, 1, 2]));
        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z];

        info!("Triangle Positions: {:?}", positions);

        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}

// HollowTriangle
#[derive(Hash)]
pub struct HollowTriangle {
    pub a: (i16, i16, i16),
    pub b: (i16, i16, i16),
    pub c: (i16, i16, i16),
}

impl AssetHash<CpuMesh> for HollowTriangle {}

impl HollowTriangle {
    pub fn new_2d(a: Vec2, b: Vec2, c: Vec2) -> Self {
        let a = a.extend(0.0);
        let b = b.extend(0.0);
        let c = c.extend(0.0);

        Self {
            a: (a.x as i16, a.y as i16, a.z as i16),
            b: (b.x as i16, b.y as i16, b.z as i16),
            c: (c.x as i16, c.y as i16, c.z as i16),
        }
    }

    pub fn new_2d_equilateral() -> Self {
        Self::new_2d(
            Vec2::new(0.0, -1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(1.0, 1.0),
        )
    }
}

impl From<HollowTriangle> for CpuMesh {
    fn from(tri: HollowTriangle) -> Self {
        let mut outer_a = Vec3::new(tri.a.0 as f32, tri.a.1 as f32, tri.a.2 as f32);
        let mut outer_b = Vec3::new(tri.b.0 as f32, tri.b.1 as f32, tri.b.2 as f32);
        let mut outer_c = Vec3::new(tri.c.0 as f32, tri.c.1 as f32, tri.c.2 as f32);

        let center = Vec3::new(
            (outer_a.x + outer_b.x + outer_c.x) / 3.0,
            (outer_a.y + outer_b.y + outer_c.y) / 3.0,
            (outer_a.z + outer_b.z + outer_c.z) / 3.0,
        );
        outer_a -= center;
        outer_b -= center;
        outer_c -= center;

        let thickness = 0.5;

        let inner_a = Vec3::new(
            outer_a.x * thickness,
            outer_a.y * thickness,
            outer_a.z * thickness,
        );
        let inner_b = Vec3::new(
            outer_b.x * thickness,
            outer_b.y * thickness,
            outer_b.z * thickness,
        );
        let inner_c = Vec3::new(
            outer_c.x * thickness,
            outer_c.y * thickness,
            outer_c.z * thickness,
        );

        let positions = vec![outer_a, outer_b, outer_c, inner_a, inner_b, inner_c];

        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z];

        let indices: Indices = Indices(Some(vec![
            0u16, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 2, 0, 3, 2, 3, 5,
        ]));

        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
