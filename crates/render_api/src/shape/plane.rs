use crate::base::{Positions, TriMesh, Vec3};
use crate::Transform;

pub struct Plane {
    pub size: f32,
}

impl Plane {
    pub fn from_size(size: f32) -> Self {
        Self { size }
    }
}

impl From<Plane> for TriMesh {
    fn from(plane: Plane) -> Self {
        let size = plane.size;
        let half_size = size / 2.0;

        let mut tri_mesh = Self::square();

        let Positions::F32(positions) = &mut tri_mesh.positions;
        for vertex in positions {
            vertex.x *= half_size;
            vertex.y *= half_size;
            vertex.z *= half_size;
        }

        tri_mesh
            .transform(&Transform::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), 90.0).to_mat4())
            .unwrap();

        tri_mesh
    }
}
