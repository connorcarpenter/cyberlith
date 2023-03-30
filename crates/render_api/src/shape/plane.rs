use crate::base::{Positions, TriMesh};

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

        let Positions::F32(positions) = &mut tri_mesh.positions else {
            panic!("Should not happen");
        };
        for vertex in positions {
            vertex.x *= half_size;
            vertex.y *= half_size;
            vertex.z *= half_size;
        }

        tri_mesh
    }
}
