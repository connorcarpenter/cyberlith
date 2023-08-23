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
        let a_x = f32::to_radians(90.0).cos();
        let a_y = f32::to_radians(90.0).sin();
        let b_x = f32::to_radians(210.0).cos();
        let b_y = f32::to_radians(210.0).sin();
        let c_x = f32::to_radians(330.0).cos();
        let c_y = f32::to_radians(330.0).sin();

        Self::new_2d(
            Vec2::new(a_x, a_y),
            Vec2::new(b_x, b_y),
            Vec2::new(c_x, c_y),
        )
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
        let a_x = f32::to_radians(90.0).cos();
        let a_y = f32::to_radians(90.0).sin();
        let b_x = f32::to_radians(210.0).cos();
        let b_y = f32::to_radians(210.0).sin();
        let c_x = f32::to_radians(330.0).cos();
        let c_y = f32::to_radians(330.0).sin();

        Self::new_2d(
            Vec2::new(a_x, a_y),
            Vec2::new(b_x, b_y),
            Vec2::new(c_x, c_y),
        )
    }
}

impl From<HollowTriangle> for CpuMesh {
    fn from(tri: HollowTriangle) -> Self {

        let outer_a = Vec3::new(tri.a.0 as f32, tri.a.1 as f32, tri.a.2 as f32);
        let outer_b = Vec3::new(tri.b.0 as f32, tri.b.1 as f32, tri.b.2 as f32);
        let outer_c = Vec3::new(tri.c.0 as f32, tri.c.1 as f32, tri.c.2 as f32);

        let center = Vec3::new(
            (outer_a.x + outer_b.x + outer_c.x) / 3.0,
            (outer_a.y + outer_b.y + outer_c.y) / 3.0,
            (outer_a.z + outer_b.z + outer_c.z) / 3.0,
        );

        let thickness = 0.9;
        let thickness_inv = 1.0 - thickness;

        let inner_a = Vec3::new(
            (outer_a.x * thickness) + (center.x * thickness_inv),
            (outer_a.y * thickness) + (center.y * thickness_inv),
            (outer_a.z * thickness) + (center.z * thickness_inv),
        );
        let inner_b = Vec3::new(
            (outer_b.x * thickness) + (center.x * thickness_inv),
            (outer_b.y * thickness) + (center.y * thickness_inv),
            (outer_b.z * thickness) + (center.z * thickness_inv),
        );
        let inner_c = Vec3::new(
            (outer_c.x * thickness) + (center.x * thickness_inv),
            (outer_c.y * thickness) + (center.y * thickness_inv),
            (outer_c.z * thickness) + (center.z * thickness_inv),
        );

        let positions = vec![
            outer_a,
            outer_b,
            outer_c,
            inner_a,
            inner_b,
            inner_c,
        ];

        let normals = vec![Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z];

        let indices: Indices = Indices(Some(vec![
            0u16, 1, 4,
            0, 4, 3,
            1, 2, 5,
            1, 5, 4,
            2, 0, 3,
            2, 3, 5,
        ]));


        CpuMesh {
            indices,
            positions: Positions(positions),
            normals: Some(normals),
            ..Default::default()
        }
    }
}
